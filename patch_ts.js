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

  // Remove unused React
  if (content.match(/import React, \{?[^}]*\}? from "react";/)) {
     content = content.replace(/import React, \{?([^}]*)\}? from "react";/, (match, p1) => {
        if (!p1.trim()) return '';
        return `import { ${p1.trim()} } from "react";`;
     });
     changed = true;
  }
  if (content.match(/import React from "react";\r?\n/)) {
     content = content.replace(/import React from "react";\r?\n/, '');
     changed = true;
  }

  // MasterDetailLayout generic parameter removal is already done, but let's check
  if (file.includes('MasterDetailLayout.tsx')) {
    if (content.includes('export function MasterDetailLayout<T extends { id: string }>')) {
      content = content.replace('export function MasterDetailLayout<T extends { id: string }>', 'export function MasterDetailLayout<T>');
      changed = true;
    }
  }

  // add idSelector to Agents, Entities, Resources, Tools, IdentityNetwork
  if (file.endsWith('Agents.tsx') || file.endsWith('Entities.tsx') || file.endsWith('Resources.tsx') || file.endsWith('Tools.tsx') || file.endsWith('IdentityNetwork.tsx')) {
    if (!content.includes('idSelector={')) {
      let idField = 'id';
      if (file.endsWith('Entities.tsx')) idField = 'entity_id';
      if (file.endsWith('Resources.tsx')) idField = 'resource_id';
      if (file.endsWith('Tools.tsx')) idField = 'tool_id';
      if (file.endsWith('IdentityNetwork.tsx')) idField = 'identity_id';
      
      content = content.replace(/<MasterDetailLayout\s+items=\{/g, `<MasterDetailLayout\n        idSelector={(x: any) => x.${idField} || x.id}\n        items={`);
      changed = true;
    }
  }

  if (changed) {
    fs.writeFileSync(file, content);
  }
}

// Custom fixes
const fixAutoDisc = () => {
  const p = path.join(srcDir, 'pages/AutoDiscovery.tsx');
  let c = fs.readFileSync(p, 'utf-8');
  c = c.replace(/import \{ Search, ShieldAlert, CheckCircle, Info, FileKey, Activity, Play \} from "lucide-react";/, 'import { Search, ShieldAlert, Info, Activity, Play } from "lucide-react";');
  fs.writeFileSync(p, c);
};
fixAutoDisc();

const fixApp = () => {
  const p = path.join(srcDir, 'App.tsx');
  let c = fs.readFileSync(p, 'utf-8');
  c = c.replace(/import \{ NAV \} from "\.\/config\/navigation";\r?\n/, '');
  fs.writeFileSync(p, c);
}
fixApp();

const fixNav = () => {
  const p = path.join(srcDir, 'config/navigation.ts');
  let c = fs.readFileSync(p, 'utf-8');
  c = c.replace(/AppMode/g, 'ProductMode');
  fs.writeFileSync(p, c);
}
fixNav();

const fixDecLogs = () => {
  const p = path.join(srcDir, 'pages/DecisionLogs.tsx');
  let c = fs.readFileSync(p, 'utf-8');
  c = c.replace(/if \(!\(await confirm\("Are you sure you want to drop all logs\?"\)\)\)/, 'if (!(await confirm({ title: "Confirm", description: "Are you sure you want to drop all logs?", danger: true })))');
  fs.writeFileSync(p, c);
}
fixDecLogs();

const fixRes = () => {
  const p = path.join(srcDir, 'pages/Resources.tsx');
  let c = fs.readFileSync(p, 'utf-8');
  c = c.replace(/if \(!\(await confirm\("Are you sure you want to delete this resource\?"\)\)\)/, 'if (!(await confirm({ title: "Confirm", description: "Are you sure you want to delete this resource?", danger: true })))');
  fs.writeFileSync(p, c);
}
fixRes();

