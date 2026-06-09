import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
    vus: 50,
    duration: '30s',
};

export default function () {
    const url = 'http://127.0.0.1:43889/v1/decision/check';
    const payload = JSON.stringify({
        request_id: `req-${__VU}-${__ITER}`,
        tenant_id: 'tenant-A',
        device_id: 'dev-001',
        action: 'read',
        resource: 'document'
    });

    const params = {
        headers: {
            'Content-Type': 'application/json',
        },
    };

    const res = http.post(url, payload, params);
    
    check(res, {
        'is status 200': (r) => r.status === 200,
        'latency < 50ms': (r) => r.timings.duration < 50,
    });
    
    sleep(0.01);
}
