import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
    vus: 50,
    duration: '30s',
};

export default function () {
    const url = 'http://127.0.0.1:43891/v1/tenants/tenant-A/devices/dev-001/bundles/metadata/targets.json';

    const res = http.get(url);
    
    check(res, {
        'is status 200': (r) => r.status === 200,
        'latency < 100ms': (r) => r.timings.duration < 100,
    });
    
    sleep(0.05);
}
