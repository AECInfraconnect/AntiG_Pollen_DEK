# คู่มือการติดตั้ง Pollen DEK (v1.0.0-beta)

## ความต้องการของระบบ

- ระบบปฏิบัติการ: Windows 10/11, macOS 12+, หรือ Ubuntu 20.04+
- พื้นที่จัดเก็บ: ว่าง 100MB
- สิทธิ์: ต้องการสิทธิ์ Administrator/root

## การติดตั้งบน Windows

1. ดาวน์โหลด `pollen-dek-x86_64-pc-windows-msvc.msi` จากหน้า GitHub Releases
2. ดับเบิลคลิกไฟล์ MSI เพื่อติดตั้งตามขั้นตอน
3. Service ชื่อ `PollenDEKCore` จะถูกติดตั้งและเริ่มทำงานโดยอัตโนมัติ

## การติดตั้งบน Linux

1. ดาวน์โหลดไฟล์ `.deb` ให้ตรงกับสถาปัตยกรรม (เช่น `pollen-dek-x86_64-unknown-linux-gnu.deb` หรือ `aarch64`)
2. รันคำสั่งติดตั้ง: `sudo dpkg -i pollen-dek-*.deb`
3. systemd service ชื่อ `pollen-dek.service` จะเริ่มทำงานโดยอัตโนมัติ

## การติดตั้งบน macOS

1. ดาวน์โหลดไฟล์ `.pkg` (เช่น `pollen-dek-x86_64-apple-darwin.pkg`)
2. รันตัวติดตั้ง package
3. launchd agent ชื่อ `ai.pollen.dek` จะโหลดและทำงานโดยอัตโนมัติ

## การตรวจสอบ

รันคำสั่ง `pollen-dekctl status` เพื่อตรวจสอบสถานะการติดตั้งและการทำงานของ service
