# คู่มือการใช้งาน Pollen DEK

## ภาพรวม
Pollen DEK (Distributed Enforcement Kernel) คือเครื่องมือสำหรับรักษาความปลอดภัยระดับ endpoint และบังคับใช้นโยบาย (policy enforcement)

## ส่วนประกอบสำคัญ
- **DEK Core**: เซอร์วิสเบื้องหลังที่จัดการการยืนยันตัวตน ดาวน์โหลดนโยบาย และควบคุมการบังคับใช้
- **DEK MCP Proxy**: พร็อกซีสำหรับการใช้งาน Model Context Protocol (MCP) ช่วยตรวจสอบสิทธิ์ก่อนส่งคำขอไปยังเครื่องมือต่างๆ
- **DEK Updater**: ระบบอัปเดตอัตโนมัติที่จะตรวจสอบ metadata และลายเซ็นดิจิทัลตามมาตรฐาน TUF ก่อนติดตั้ง

## การตั้งค่า
ไฟล์การตั้งค่าจะอยู่ที่ `/opt/pollen` (Linux), `/Library/Application Support/PollenDEK` (macOS), หรือ `C:\ProgramData\PollenDEK` (Windows)

## บันทึกการทำงาน (Logs)
Logs จะอยู่ในโฟลเดอร์ `logs` ภายใต้โฟลเดอร์การตั้งค่า
