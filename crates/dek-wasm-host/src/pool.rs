use crate::{compiled_plugin::CompiledPlugin, worker::PluginWorker};
use anyhow::Result;
use parking_lot::Mutex;
use std::{collections::VecDeque, sync::Arc, time::Duration};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use wasmtime::Engine;

pub struct WorkerLease {
    worker: Option<PluginWorker>,
    _permit: OwnedSemaphorePermit,
}

impl WorkerLease {
    pub fn worker_mut(&mut self) -> anyhow::Result<&mut PluginWorker> {
        self.worker
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("worker lease already released"))
    }

    fn take(mut self) -> anyhow::Result<PluginWorker> {
        self.worker
            .take()
            .ok_or_else(|| anyhow::anyhow!("worker lease already released"))
    }
}

pub struct PluginWorkerPool {
    engine: Arc<Engine>,
    compiled: Arc<CompiledPlugin>,
    generation: u64,
    min_warm: usize,
    max_warm: usize,
    max_worker_uses: u64,
    invoke_timeout: Duration,
    max_memory_bytes: usize,
    table_elements: u32,
    warm: Mutex<VecDeque<PluginWorker>>,
    semaphore: Arc<Semaphore>,
}

impl PluginWorkerPool {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        engine: Arc<Engine>,
        compiled: Arc<CompiledPlugin>,
        generation: u64,
        min_warm: usize,
        max_warm: usize,
        max_concurrency: usize,
        max_worker_uses: u64,
        invoke_timeout: Duration,
        max_memory_bytes: usize,
        table_elements: u32,
    ) -> Self {
        Self {
            engine,
            compiled,
            generation,
            min_warm,
            max_warm,
            max_worker_uses,
            invoke_timeout,
            max_memory_bytes,
            table_elements,
            warm: Mutex::new(VecDeque::new()),
            semaphore: Arc::new(Semaphore::new(max_concurrency)),
        }
    }

    pub async fn prewarm(&self) -> Result<()> {
        for i in 0..self.min_warm {
            let worker = PluginWorker::new(
                &self.engine,
                &self.compiled,
                self.generation,
                format!("prewarm-{i}"),
                self.invoke_timeout,
                self.max_memory_bytes,
                self.table_elements,
            )
            .await?;
            self.warm.lock().push_back(worker);
        }
        Ok(())
    }

    pub async fn acquire(
        &self,
        request_id: String,
        acquire_timeout: Duration,
    ) -> Result<WorkerLease> {
        let permit = tokio::time::timeout(acquire_timeout, self.semaphore.clone().acquire_owned())
            .await
            .map_err(|_| anyhow::anyhow!("timeout acquiring plugin concurrency permit"))??;

        if let Some(worker) = self.warm.lock().pop_front() {
            return Ok(WorkerLease {
                worker: Some(worker),
                _permit: permit,
            });
        }

        // Pool miss: instantiate a new worker. With InstancePre and pooling allocator,
        // this should be much cheaper than full cold start.
        let worker = PluginWorker::new(
            &self.engine,
            &self.compiled,
            self.generation,
            request_id,
            self.invoke_timeout,
            self.max_memory_bytes,
            self.table_elements,
        )
        .await?;

        Ok(WorkerLease {
            worker: Some(worker),
            _permit: permit,
        })
    }

    pub async fn release(&self, lease: WorkerLease) -> Result<()> {
        let mut worker = lease.take()?;

        if worker.generation != self.generation {
            return Ok(()); // old generation; drop
        }

        if worker.uses >= self.max_worker_uses {
            return Ok(()); // rotate worker to reduce long-lived state risk
        }

        let reusable = worker.reset_for_reuse().await.unwrap_or(false);
        if !reusable {
            return Ok(()); // discard dirty/faulted worker
        }

        let mut warm = self.warm.lock();
        if warm.len() < self.max_warm {
            warm.push_back(worker);
        }

        Ok(())
    }
}
