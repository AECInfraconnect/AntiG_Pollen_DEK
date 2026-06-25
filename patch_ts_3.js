const fs = require('fs');
const path = require('path');

const srcDir = path.join(__dirname, 'apps/local-admin-dashboard/src');

const filesToClean = [
  'components/master-detail/DetailPane.tsx',
  'components/master-detail/EntityCard.tsx',
  'pages/Agents.tsx',
  'pages/BlackboxAI.tsx',
  'pages/Data/IdentityNetwork.tsx',
  'pages/DecisionLogs.tsx',
  'pages/Entities.tsx',
  'pages/Resources.tsx',
  'pages/Tools.tsx',
  'config/navigation.ts',
  'components/simple/SimplePolicyWizard.tsx'
];

for (const file of filesToClean) {
  const p = path.join(srcDir, file);
  if (fs.existsSync(p)) {
    let c = fs.readFileSync(p, 'utf-8');
    
    // fix import { statusToken, type UiStatus } in DetailPane
    if (file.includes('DetailPane.tsx')) {
       c = c.replace(/import \{ statusToken, type UiStatus \} from "\.\.\/\.\.\/lib\/status";/, 'import type { UiStatus } from "../../lib/status";');
    }
    
    if (file.includes('EntityCard.tsx')) {
       c = c.replace(/import\s*\{\s*UiStatus\s*\}\s*from/, 'import type { UiStatus } from');
    }

    if (file.includes('Agents.tsx') || file.includes('BlackboxAI.tsx') || file.includes('Tools.tsx')) {
       c = c.replace(/import \{ useConfirm \} from "\.\.\/components\/ui\/ConfirmDialog";\r?\n/, '');
    }

    if (file.includes('DecisionLogs.tsx') || file.includes('Resources.tsx')) {
       c = c.replace(/await confirm\(".*?"\)/g, 'await confirm({title: "Confirm", description: "Are you sure?", danger: true})');
    }

    if (file.includes('Entities.tsx')) {
       c = c.replace(/import \{ CardSkeleton \} from "\.\.\/components\/master-detail\/CardSkeleton";\r?\n/, '');
       c = c.replace(/const \{ confirm \} = useConfirm\(\);\r?\n/, '');
    }

    if (file.includes('Tools.tsx')) {
       c = c.replace(/JSON\.stringify\(c\.schema, null, 2\)/g, 'JSON.stringify((c as any).schema, null, 2)');
    }

    if (file.includes('IdentityNetwork.tsx')) {
       c = c.replace(/'\.\.\/services\/api'/g, "'../../services/api'");
       c = c.replace(/'\.\.\/services\/types'/g, "'../../services/types'");
       c = c.replace(/import type \{ UiStatus \} from "\.\.\/lib\/status";\r?\n/, '');
    }

    if (file.includes('navigation.ts')) {
       c = c.replace(/AppMode/g, 'string');
    }

    if (file.includes('SimplePolicyWizard.tsx')) {
       c = c.replace(/mode === "enterprise_server"/g, 'mode === "enterprise"');
    }

    fs.writeFileSync(p, c);
  }
}
