import { useState, useEffect } from "react";
import { SimplePolicyWizard } from "../components/simple/SimplePolicyWizard";
import { RegistryApi } from "../services/api";

export function Wizard() {
  const [agents, setAgents] = useState<{ id: string; label: string }[]>([]);
  useEffect(() => {
    RegistryApi.listAgents().then((data) => {
      setAgents(data.map((a) => ({ id: a.agent_id, label: a.name })));
    });
  }, []);
  return <SimplePolicyWizard agents={agents} />;
}
