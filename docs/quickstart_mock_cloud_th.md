# เริ่มต้นการใช้งาน Mock-Cloud Sandbox

Pollek Local Enforcement Kit Mock-Cloud คือระบบจำลองที่ให้คุณสามารถทดสอบการเชื่อมต่อและการทำงานของ Local Enforcement Kit ในระบบปิดได้โดยไม่ต้องใช้ระบบคลาวด์จริง

## ขั้นตอนการเริ่มใช้งานแบบ Quickstart

ทำตาม 6 ขั้นตอนนี้เพื่อจำลองการทำงานของระบบให้สมบูรณ์:

### 1. เริ่มรัน Mock Cloud

```bash
Pollek-mock-cloud --seed sandbox
```

### 2. ลงทะเบียน Local Enforcement Kit (Enroll)

```bash
Pollek-dekctl enroll --cloud-url https://127.0.0.1:43892
```

### 3. รัน Local Enforcement Kit

```bash
Pollek-Local Enforcement Kit --config ~/.Pollek/Local Enforcement Kit/bootstrap.json
```

### 4. รัน MCP Proxy

```bash
Pollek-mcp-proxy --listen 127.0.0.1:8787
```

### 5. รันสคริปต์ทดสอบ Test Client

```bash
Pollek-dekctl sandbox run mcp-allow-deny
```

### 6. ดู Telemetry

เข้าดู Dashboard ได้ที่: `https://127.0.0.1:43892/admin/dashboard` (หรือ `http://127.0.0.1:43892` ขึ้นอยู่กับการตั้งค่า Mock-Cloud)
คุณสามารถ:

- สั่งจำลองการอัปเดต Bundle และแก้ไขนโยบาย
- สั่งจำลองระบบเครือข่ายล่มหรือเกิดเหตุการณ์ป่วน (Chaos events)
- หมุนเวียนกุญแจเข้ารหัสและเพิกถอนสิทธิ์อุปกรณ์
- ตรวจสอบประวัติการเข้าถึงและการอนุญาต (Decisions and Trace logs)
