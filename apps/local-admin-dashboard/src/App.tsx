import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import { DashboardLayout } from "./components/layout/DashboardLayout";
import { Overview } from "./pages/Overview";
import { Agents } from "./pages/Agents";
import { Servers } from "./pages/Servers";
import { Tools } from "./pages/Tools";
import { Resources } from "./pages/Resources";
import { Policies } from "./pages/Policies";
import { DecisionLogs } from "./pages/DecisionLogs";

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
          <Route path="relationships" element={<div className="glass p-6 rounded-xl"><h1>Relationships (WIP)</h1></div>} />
          <Route path="policies" element={<Policies />} />
          <Route path="audit" element={<DecisionLogs />} />
          <Route path="alerts" element={<div className="glass p-6 rounded-xl"><h1>Alerts (WIP)</h1></div>} />
        </Route>
      </Routes>
    </Router>
  );
}

export default App;
