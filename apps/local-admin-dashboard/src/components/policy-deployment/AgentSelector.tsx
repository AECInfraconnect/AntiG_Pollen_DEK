

export function AgentSelector() {
  return (
    <div className="space-y-4">
      <h4 className="font-medium">Select Target Agents</h4>
      <div className="p-4 border rounded bg-muted/20">
        <label className="flex items-center gap-2">
          <input type="checkbox" checked readOnly />
          <span>All Compatible Agents</span>
        </label>
      </div>
    </div>
  );
}
