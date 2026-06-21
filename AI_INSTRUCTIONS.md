# AI Assistant Instructions

**CRITICAL RULE FOR ALL AI ASSISTANTS WORKING ON THIS REPOSITORY:**

Whenever you complete a major task, phase, or milestone, you **MUST** automatically commit and push the changes to GitHub.

Use the following sequence of commands (in PowerShell):
```powershell
& "C:\Program Files\Git\cmd\git.exe" add .
& "C:\Program Files\Git\cmd\git.exe" commit -m "feat: <description of your work>"
& "C:\Program Files\Git\cmd\git.exe" push origin main
```

Do not wait for the user to remind you. Proactively ask for confirmation to push or simply push if the user has given blanket approval for the task.
