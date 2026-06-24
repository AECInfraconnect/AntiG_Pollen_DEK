import { useState } from "react";
import { useNavigate } from "react-router-dom";

export function Wizard() {
  const [step, setStep] = useState(1);
  const navigate = useNavigate();

  const handleAgree = async () => {
    // In a real app, this would hit the dek-core API to record consent
    console.log("Consent recorded");
    setStep(2);
  };

  const handleProfile = async (profile: "local" | "cloud") => {
    // Hit API to set profile
    console.log(`Profile set to ${profile}`);
    setStep(3);
  };

  const handleComplete = () => {
    navigate("/");
  };

  return (
    <div className="min-h-screen bg-gray-900 text-white flex flex-col items-center justify-center p-4">
      <div className="max-w-2xl w-full bg-gray-800 rounded-xl p-8 shadow-2xl border border-gray-700">
        <div className="flex justify-between items-center mb-8 border-b border-gray-700 pb-4">
          <h1 className="text-2xl font-bold bg-gradient-to-r from-blue-400 to-indigo-400 bg-clip-text text-transparent">
            Pollen DEK Onboarding
          </h1>
          <div className="text-sm text-gray-400">Step {step} of 3</div>
        </div>

        {step === 1 && (
          <div className="space-y-6 animate-in fade-in slide-in-from-bottom-4">
            <h2 className="text-xl font-semibold">Agreements & Privacy</h2>
            <div className="bg-gray-900 p-4 rounded-lg border border-gray-700 text-sm text-gray-300 h-48 overflow-y-auto">
              <p className="mb-4">
                Welcome to Pollen DEK. To proceed, you must agree to our End
                User License Agreement (EULA) and Privacy Notice.
              </p>
              <p className="mb-4">
                Pollen DEK processes your data locally by default. Telemetry and
                browser history scans are strictly opt-in and require explicit
                consent to enable advanced shadow AI discovery.
              </p>
              <p>
                By clicking "I Agree", you acknowledge that you have read and
                understood the agreements.
              </p>
            </div>
            <div className="flex justify-end pt-4">
              <button
                onClick={handleAgree}
                className="px-6 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg transition-colors shadow-lg"
              >
                I Agree & Continue
              </button>
            </div>
          </div>
        )}

        {step === 2 && (
          <div className="space-y-6 animate-in fade-in slide-in-from-bottom-4">
            <h2 className="text-xl font-semibold">Select Operating Mode</h2>
            <p className="text-gray-400 text-sm">
              Choose how Pollen DEK connects to the control plane. This can be
              changed later.
            </p>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <button
                onClick={() => handleProfile("local")}
                className="p-6 bg-gray-900 border border-gray-700 hover:border-blue-500 rounded-xl text-left transition-all hover:shadow-[0_0_15px_rgba(59,130,246,0.3)] group"
              >
                <div className="flex items-center gap-3 mb-2">
                  <div className="w-8 h-8 rounded bg-blue-500/20 flex items-center justify-center text-blue-400">
                    <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                    </svg>
                  </div>
                  <h3 className="font-semibold text-lg group-hover:text-blue-400 transition-colors">Sovereign Mode</h3>
                </div>
                <p className="text-xs text-gray-400 leading-relaxed">
                  Fully air-gapped. 100% local enforcement. No telemetry or
                  policies are sent to the cloud. Best for highly regulated
                  environments.
                </p>
              </button>

              <button
                onClick={() => handleProfile("cloud")}
                className="p-6 bg-gray-900 border border-gray-700 hover:border-purple-500 rounded-xl text-left transition-all hover:shadow-[0_0_15px_rgba(168,85,247,0.3)] group"
              >
                <div className="flex items-center gap-3 mb-2">
                  <div className="w-8 h-8 rounded bg-purple-500/20 flex items-center justify-center text-purple-400">
                    <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 15a4 4 0 004 4h9a5 5 0 10-.1-9.999 5.002 5.002 0 10-9.78 2.096A4.001 4.001 0 003 15z" />
                    </svg>
                  </div>
                  <h3 className="font-semibold text-lg group-hover:text-purple-400 transition-colors">Cloud Managed</h3>
                </div>
                <p className="text-xs text-gray-400 leading-relaxed">
                  Connect to Pollen Cloud. Receive real-time policy updates,
                  threat intelligence, and central audit logging.
                </p>
              </button>
            </div>
          </div>
        )}

        {step === 3 && (
          <div className="space-y-6 text-center animate-in fade-in slide-in-from-bottom-4 py-8">
            <div className="w-20 h-20 bg-green-500/20 text-green-400 rounded-full flex items-center justify-center mx-auto mb-6 shadow-[0_0_30px_rgba(34,197,94,0.3)]">
              <svg className="w-10 h-10" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
              </svg>
            </div>
            <h2 className="text-2xl font-bold">You're all set!</h2>
            <p className="text-gray-400">
              Pollen DEK is now protecting your environment. We will run an
              initial background scan to discover existing AI agents.
            </p>
            <div className="pt-6">
              <button
                onClick={handleComplete}
                className="px-8 py-3 bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-500 hover:to-indigo-500 text-white rounded-lg font-medium transition-all shadow-lg hover:shadow-indigo-500/25"
              >
                Go to Dashboard
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
