# คู่มือการใช้งาน Pollen DEK

## ภาพรวม

Pollen DEK (Distributed Enforcement Kernel) คือเครื่องมือสำหรับรักษาความปลอดภัยระดับ endpoint และบังคับใช้นโยบาย (policy enforcement)

## ส่วนประกอบสำคัญ

- **Pollen DEK Core (`pollen-dek`)**: เซอร์วิสเบื้องหลังที่จัดการการยืนยันตัวตน ดาวน์โหลดนโยบาย และควบคุมการบังคับใช้
- **Pollen DEK CLI (`pollen-dekctl`)**: เครื่องมือ Command-line สำหรับลงทะเบียน จัดการ และตรวจสอบการทำงานของ DEK
- **Pollen MCP Proxy (`pollen-mcp-proxy`)**: พร็อกซีสำหรับการใช้งาน Model Context Protocol (MCP) ช่วยตรวจสอบสิทธิ์ก่อนส่งคำขอไปยังเครื่องมือต่างๆ
- **Mock Cloud (`pollen-mock-cloud`)**: ระบบจำลองการทำงานของ Pollen Cloud สำหรับการพัฒนาและทดสอบในช่วง Beta

## การตั้งค่า

ในช่วงทดสอบ Beta ไฟล์การตั้งค่าจะอยู่ที่ `~/.pollen/dek/` โดยค่าเริ่มต้น ซึ่งจะใช้ไฟล์ `bootstrap.json`

## บันทึกการทำงาน (Logs)

สามารถดู Logs ได้โดยใช้คำสั่ง `pollen-dekctl logs` หรือเปิดดูไฟล์ในโฟลเดอร์ `~/.pollen/dek/logs/`
