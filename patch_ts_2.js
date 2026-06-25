const fs = require('fs');
const path = require('path');

const srcDir = path.join(__dirname, 'apps/local-admin-dashboard/src');
const walk = (dir) => {
  let results = [];
  const list = fs.readdirSync(dir);
  list.forEach((file) => {
    file = path.join(dir, file);
    const stat = fs.statSync(file);
    if (stat && stat.isDirectory()) {
      results = results.concat(walk(file));
    } else {
      if (file.endsWith('.tsx') || file.endsWith('.ts')) {
        results.push(file);
      }
    }
  });
  return results;
};

const files = walk(srcDir);

for (const file of files) {
  let content = fs.readFileSync(file, 'utf-8');
  let changed = false;

  if (content.includes('import { UiStatus }') || content.includes('import { UiStatus,')) {
    content = content.replace(/import\s*\{\s*UiStatus\s*\}\s*from/, 'import type { UiStatus } from');
    content = content.replace(/import\s*\{\s*UiStatus\s*,\s*statusToken\s*\}\s*from/, 'import { statusToken, type UiStatus } from');
    changed = true;
  }
  
  if (content.includes('import { ReactNode }')) {
    content = content.replace('import { ReactNode }', 'import type { ReactNode }');
    changed = true;
  }
  if (content.includes('import { createContext, useContext, useState, ReactNode }')) {
    content = content.replace('import { createContext, useContext, useState, ReactNode }', 'import { createContext, useContext, useState, type ReactNode }');
    changed = true;
  }

  // Use confirm
  if (content.includes('const { confirm } = useConfirm();')) {
     if (content.match(/import \{ useConfirm \} from ".*?";\r?\n\r?\nexport function/)) {
        content = content.replace(/import \{ useConfirm \} from ".*?";\r?\n/, '');
     }
  }

  if (changed) fs.writeFileSync(file, content);
}

// Data/IdentityNetwork
let idNet = fs.readFileSync(path.join(srcDir, 'pages/Data/IdentityNetwork.tsx'), 'utf-8');
idNet = idNet.replace(/'\.\.\/services\/api'/g, "'../../services/api'");
idNet = idNet.replace(/'\.\.\/services\/types'/g, "'../../services/types'");
idNet = idNet.replace(/import type \{ UiStatus \} from "\.\.\/lib\/status";\r?\n/, 'import type { UiStatus } from "../../lib/status";\n');
fs.writeFileSync(path.join(srcDir, 'pages/Data/IdentityNetwork.tsx'), idNet);

// DecisionLogs
let decLogs = fs.readFileSync(path.join(srcDir, 'pages/DecisionLogs.tsx'), 'utf-8');
decLogs = decLogs.replace(/await confirm\("Are you sure you want to drop all logs\?"\)/g, 'await confirm({title: "Confirm", description: "Are you sure you want to drop all logs?", danger: true})');
fs.writeFileSync(path.join(srcDir, 'pages/DecisionLogs.tsx'), decLogs);

// config/navigation.ts
let nav = fs.readFileSync(path.join(srcDir, 'config/navigation.ts'), 'utf-8');
nav = nav.replace(/ProductMode/g, 'AppMode');
fs.writeFileSync(path.join(srcDir, 'config/navigation.ts'), nav);

// context/ModeContext.tsx
let modeCtx = fs.readFileSync(path.join(srcDir, 'context/ModeContext.tsx'), 'utf-8');
modeCtx = modeCtx.replace(/import type \{ ProductMode \} from "\.\.\/navigation\/menu";/g, 'export type AppMode = "desktop_simple" | "desktop_advanced" | "enterprise";');
modeCtx = modeCtx.replace(/ProductMode/g, 'AppMode');
fs.writeFileSync(path.join(srcDir, 'context/ModeContext.tsx'), modeCtx);

// pages/Tools.tsx schema fix
let tools = fs.readFileSync(path.join(srcDir, 'pages/Tools.tsx'), 'utf-8');
tools = tools.replace(/JSON\.stringify\(c\.schema, null, 2\)/g, 'JSON.stringify((c as any).schema, null, 2)');
fs.writeFileSync(path.join(srcDir, 'pages/Tools.tsx'), tools);
