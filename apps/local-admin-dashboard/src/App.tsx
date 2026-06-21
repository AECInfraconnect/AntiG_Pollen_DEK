import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import { DashboardLayout } from "./components/layout/DashboardLayout";
import { Overview } from "./pages/Overview";
import { Agents } from "./pages/Agents";
import { Servers } from "./pages/Servers";
import { Tools } from "./pages/Tools";
import { Resources } from "./pages/Resources";
import { Policies } from "./pages/Policies";
import { Simulator } from "./pages/Simulator";
import { Bundles } from "./pages/Bundles";
import { DecisionLogs } from "./pages/DecisionLogs";
import { Entities } from "./pages/Entities";
import { Relationships } from "./pages/Relationships";
import { BlackboxAI } from "./pages/BlackboxAI";
import { Settings } from "./pages/Settings";

function App() {
  return (
    <Router>
      <Routes>
        <Route path="/" element={<DashboardLayout />}>
          <Route index element={<Overview />} />
          <Route path="agents" element={<Agents />} />
          <Route path="servers" element={<Servers />} />
          <Route path="tools" element={<Tools />} />
          <Route path="resources" element={<Resources />} />
          <Route path="entities" element={<Entities />} />
          <Route path="relationships" element={<Relationships />} />
          <Route path="blackbox-ai" element={<BlackboxAI />} />
          <Route path="policies" element={<Policies />} />
          <Route path="simulator" element={<Simulator />} />
          <Route path="bundles" element={<Bundles />} />
          <Route path="audit" element={<DecisionLogs />} />
          <Route path="alerts" element={<div className="glass p-6 rounded-xl"><h1>Alerts (WIP)</h1></div>} />
          <Route path="settings" element={<Settings />} />
        </Route>
      </Routes>
    </Router>
  );
}

export default App;
