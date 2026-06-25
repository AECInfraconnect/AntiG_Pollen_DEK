const fs = require('fs');
const path = require('path');
const srcDir = path.join(__dirname, 'src');

const delStr = (p, regex, replace='') => {
  if (fs.existsSync(p)) {
    let c = fs.readFileSync(p, 'utf-8');
    fs.writeFileSync(p, c.replace(regex, replace));
  }
};

delStr(path.join(srcDir, 'config/navigation.ts'), /import type \{ AppMode \} from "\.\.\/context\/ModeContext";\r?\n/);
delStr(path.join(srcDir, 'pages/Data/IdentityNetwork.tsx'), /import \{ Users, FileKey, ShieldAlert, Zap, UserCircle, Network, Activity, Info \} from "lucide-react";\r?\n/, 'import { UserCircle, Network, Activity, Info } from "lucide-react";\n');
delStr(path.join(srcDir, 'pages/DecisionLogs.tsx'), /await confirm\("Are you sure you want to drop all logs\?"\)/g, 'await confirm({title: "Confirm", description: "Are you sure?", danger: true})');
delStr(path.join(srcDir, 'pages/Entities.tsx'), /import \{ useConfirm \} from "\.\.\/components\/ui\/ConfirmDialog";\r?\n/);
delStr(path.join(srcDir, 'pages/Resources.tsx'), /await confirm\("Are you sure you want to delete this resource\?"\)/g, 'await confirm({title: "Confirm", description: "Are you sure you want to delete this resource?", danger: true})');
delStr(path.join(srcDir, 'pages/Tools.tsx'), /JSON\.stringify\(c\.schema, null, 2\)/g, 'JSON.stringify((c as any).schema, null, 2)');
