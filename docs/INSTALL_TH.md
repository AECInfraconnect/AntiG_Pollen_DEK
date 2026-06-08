# คู่มือการติดตั้ง Pollen DEK

## ความต้องการของระบบ
- ระบบปฏิบัติการ: Windows 10/11, macOS 12+, หรือ Ubuntu 20.04+
- พื้นที่จัดเก็บ: ว่าง 100MB
- สิทธิ์: ต้องการสิทธิ์ Administrator/root

## การติดตั้งบน Windows
1. ดาวน์โหลด `pollen-dek-windows-msi` จาก Releases
2. รันตัวติดตั้ง MSI
3. DEK Core Service จะเริ่มทำงานโดยอัตโนมัติ

## การติดตั้งบน Linux
1. ดาวน์โหลด `.deb` หรือ `.tar.gz`
2. สำหรับ `.deb`: `sudo dpkg -i pollen-dek_1.0.0_amd64.deb`
3. systemd service จะเริ่มทำงานโดยอัตโนมัติ

## การติดตั้งบน macOS
1. ดาวน์โหลด `.pkg`
2. รันตัวติดตั้ง package
3. launchd agent จะโหลดโดยอัตโนมัติ

## การตรวจสอบ
รันคำสั่ง `dek-core --version` เพื่อตรวจสอบการติดตั้ง
