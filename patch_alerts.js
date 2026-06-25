const fs = require('fs');
const path = require('path');

const pagesDir = path.join(__dirname, 'apps/local-admin-dashboard/src/pages');
const files = fs.readdirSync(pagesDir).filter(f => f.endsWith('.tsx'));

for (const file of files) {
  const filePath = path.join(pagesDir, file);
  let content = fs.readFileSync(filePath, 'utf-8');
  let changed = false;

  // Add imports if needed
  if (content.includes('alert(') && !content.includes('toast')) {
    content = 'import { toast } from "sonner";\n' + content;
  }
  
  if (content.includes('confirm(') && !content.includes('useConfirm')) {
    if (!content.includes('import { useConfirm }')) {
       content = 'import { useConfirm } from "../components/ui/ConfirmDialog";\n' + content;
    }
    // Inject hook
    content = content.replace(/export function (\w+)\(\) \{/, 'export function $1() {\n  const { confirm } = useConfirm();\n');
  }

  if (content.includes('alert(') || content.includes('confirm(')) {
    content = content.replace(/alert\((['"`].+?['"`])\)/g, 'toast.error($1)');
    content = content.replace(/if \(!confirm\((['"`].+?['"`])\)\) return;/g, 'if (!(await confirm({ title: "Confirm Action", description: $1, danger: true }))) return;');
    content = content.replace(/const delete/g, 'const delete'); // Just to dirty it? No.
    changed = true;
  }

  if (changed) {
    fs.writeFileSync(filePath, content);
    console.log(`Patched ${file}`);
  }
}
