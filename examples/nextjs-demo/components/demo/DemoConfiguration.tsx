"use client";

import { useEffect, useState } from "react";
import { Settings2, RefreshCw, Info, SlidersHorizontal } from "lucide-react";
import {
  useDemoConfig,
  useDemoDispatch,
  updateConfig,
  setConfig,
} from "@/lib/demo/state";
import type { DemoConfig } from "@/lib/demo/types";

/**
 * LocalStorage key for persisting demo configuration.
 * This is intentionally scoped to the Next.js demo sandbox.
 */
const STORAGE_KEY = "poblysh-nextjs-demo-config";

/**
 * Available preset configurations for educational modes.
 */
const PRESETS: Record<
  string,
  {
    label: string;
    description: string;
    config: DemoConfig;
  }
> = {
  quick: {
    label: "Quick Demo",
    description:
      "Fast timing, minimal errors, simple providers. Great for first-time walkthroughs.",
    config: {
      signalFrequency: "low",
      errorRate: "0%",
      timingMode: "fast",
      providerComplexity: "simple",
      mode: "mock",
      isConfigValid: true,
      configErrors: [],
      configWarnings: [],
    },
  },
  realistic: {
    label: "Realistic Demo",
    description:
      "Production-like timing, moderate errors, detailed provider behavior.",
    config: {
      signalFrequency: "medium",
      errorRate: "10%",
      timingMode: "realistic",
      providerComplexity: "detailed",
      mode: "mock",
      isConfigValid: true,
      configErrors: [],
      configWarnings: [],
    },
  },
  errors: {
    label: "Error Scenarios",
    description:
      "Higher error rate and realistic timing to highlight resilience and recovery patterns.",
    config: {
      signalFrequency: "medium",
      errorRate: "20%",
      timingMode: "realistic",
      providerComplexity: "detailed",
      mode: "mock",
      isConfigValid: true,
      configErrors: [],
      configWarnings: [],
    },
  },
  performance: {
    label: "Performance Demo",
    description:
      "High signal volume with fast timing to explore performance and scaling considerations.",
    config: {
      signalFrequency: "high",
      errorRate: "10%",
      timingMode: "fast",
      providerComplexity: "detailed",
      mode: "mock",
      isConfigValid: true,
      configErrors: [],
      configWarnings: [],
    },
  },
};

interface DemoConfigurationProps {
  /**
   * Optional additional class names for layout integration.
   */
  className?: string;
}

/**
 * DemoConfiguration component
 *
 * Aligns with the OpenSpec "Configurable Demo Parameters" requirement:
 * - Exposes controls for signal frequency, error rate, timing mode, and provider complexity.
 * - Provides educational presets that map to common learning scenarios.
 * - Persists configuration to localStorage so users get a consistent experience.
 * - Uses the shared DemoState config so all generators can react accordingly.
 */
export default function DemoConfiguration({
  className = "",
}: DemoConfigurationProps) {
  const globalConfig = useDemoConfig();
  const dispatch = useDemoDispatch();

  // Local UI state is derived from globalConfig for responsive controls.
  const [config, setLocalConfig] = useState<DemoConfig>(globalConfig);
  const [activePreset, setActivePreset] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  // On mount: hydrate from localStorage if available.
  useEffect(() => {
    try {
      const stored =
        typeof window !== "undefined"
          ? window.localStorage.getItem(STORAGE_KEY)
          : null;

      if (stored) {
        const parsed = JSON.parse(stored) as DemoConfig;
        if (isValidConfig(parsed)) {
          setLocalConfig(parsed);
          dispatch(setConfig(parsed));
          // Try to infer active preset
          const presetKey = inferPresetKey(parsed);
          setActivePreset(presetKey);
          return;
        }
      }
    } catch {
      // Ignore storage errors; fall back to existing global config.
    }
    // No stored config / invalid: sync from current global config.
    setLocalConfig(globalConfig);
    setActivePreset(inferPresetKey(globalConfig));
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  // If globalConfig changes elsewhere, keep local state in sync (best-effort).
  useEffect(() => {
    setLocalConfig(globalConfig);
    setActivePreset(inferPresetKey(globalConfig));
  }, [globalConfig]);

  const handleChange = <K extends keyof DemoConfig>(
    key: K,
    value: DemoConfig[K],
  ) => {
    const next: DemoConfig = {
      ...config,
      [key]: value,
    };
    setLocalConfig(next);
    setActivePreset(inferPresetKey(next));
    persistAndDispatch(next);
  };

  const handlePresetClick = (key: string) => {
    const preset = PRESETS[key];
    if (!preset) return;
    setActivePreset(key);
    setLocalConfig(preset.config);
    persistAndDispatch(preset.config);
  };

  const handleResetDefaults = () => {
    const defaults: DemoConfig = {
      signalFrequency: "medium",
      errorRate: "10%",
      timingMode: "realistic",
      providerComplexity: "detailed",
      mode: "mock",
      isConfigValid: true,
      configErrors: [],
      configWarnings: [],
    };
    setActivePreset(inferPresetKey(defaults));
    setLocalConfig(defaults);
    persistAndDispatch(defaults, true);
  };

  const persistAndDispatch = (next: DemoConfig, isReset = false) => {
    setSaving(true);
    // Update global demo state
    dispatch(updateConfig(next));
    // Persist to localStorage if available
    try {
      if (typeof window !== "undefined") {
        if (isReset) {
          window.localStorage.removeItem(STORAGE_KEY);
        } else {
          window.localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
        }
      }
    } catch {
      // Non-fatal: persistence is best-effort for the sandbox.
    } finally {
      // Brief delay for subtle UI feedback without being noisy.
      setTimeout(() => setSaving(false), 150);
    }
  };

  return (
    <section
      className={`bg-white border border-gray-200 rounded-lg p-6 space-y-5 ${className}`}
      aria-labelledby="demo-config-heading"
    >
      {/* Header */}
      <div className="flex items-start justify-between gap-3">
        <div className="flex items-center gap-2">
          <div className="inline-flex items-center justify-center w-8 h-8 rounded-md bg-gray-900 text-white">
            <Settings2 className="w-4 h-4" />
          </div>
          <div>
            <h2
              id="demo-config-heading"
              className="text-base font-semibold text-gray-900"
            >
              Demo configuration
            </h2>
            <p className="text-xs text-gray-600">
              Tune signal volume, error rates, timing, and complexity to match
              your learning scenario. Changes apply immediately across the mock
              demo.
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2 text-[10px] text-gray-500">
          {saving ? (
            <span className="inline-flex items-center gap-1">
              <RefreshCw className="w-3 h-3 animate-spin" />
              Saving…
            </span>
          ) : (
            <span className="text-gray-400">Config synced</span>
          )}
        </div>
      </div>

      {/* Preset buttons */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-2">
        {Object.entries(PRESETS).map(([key, preset]) => (
          <button
            key={key}
            type="button"
            onClick={() => handlePresetClick(key)}
            className={`flex flex-col items-start gap-1 px-3 py-2 rounded-md border text-left transition-colors ${
              activePreset === key
                ? "border-gray-900 bg-gray-900 text-white"
                : "border-gray-200 bg-gray-50 text-gray-800 hover:bg-gray-100"
            }`}
          >
            <span className="text-xs font-semibold">{preset.label}</span>
            <span
              className={`text-[10px] leading-snug ${
                activePreset === key
                  ? "text-gray-200"
                  : "text-gray-500 line-clamp-3"
              }`}
            >
              {preset.description}
            </span>
          </button>
        ))}
      </div>

      {/* Sliders / selectors */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4 items-start">
        {/* Signal Frequency */}
        <ConfigField
          label="Signal frequency"
          description="Controls how many signals are generated per connection."
        >
          <SelectPillRow
            value={config.signalFrequency}
            onChange={(value) =>
              handleChange(
                "signalFrequency",
                value as DemoConfig["signalFrequency"],
              )
            }
            options={[
              { value: "low", label: "Low" },
              { value: "medium", label: "Medium" },
              { value: "high", label: "High" },
            ]}
          />
        </ConfigField>

        {/* Error Rate */}
        <ConfigField
          label="Error rate"
          description="Simulate errors across sync jobs, webhooks, and tokens."
        >
          <SelectPillRow
            value={config.errorRate}
            onChange={(value) =>
              handleChange("errorRate", value as DemoConfig["errorRate"])
            }
            options={[
              { value: "0%", label: "0%" },
              { value: "10%", label: "10%" },
              { value: "20%", label: "20%" },
            ]}
          />
        </ConfigField>

        {/* Timing Mode */}
        <ConfigField
          label="Timing mode"
          description="Choose between fast feedback or production-like latencies."
        >
          <SelectPillRow
            value={config.timingMode}
            onChange={(value) =>
              handleChange("timingMode", value as DemoConfig["timingMode"])
            }
            options={[
              { value: "fast", label: "Fast" },
              { value: "realistic", label: "Realistic" },
            ]}
          />
        </ConfigField>

        {/* Provider Complexity */}
        <ConfigField
          label="Provider complexity"
          description="Toggle between simple and detailed provider behaviors."
        >
          <SelectPillRow
            value={config.providerComplexity}
            onChange={(value) =>
              handleChange(
                "providerComplexity",
                value as DemoConfig["providerComplexity"],
              )
            }
            options={[
              { value: "simple", label: "Simple" },
              { value: "detailed", label: "Detailed" },
            ]}
          />
        </ConfigField>
      </div>

      {/* Actions + educational hint */}
      <div className="flex flex-col sm:flex-row sm:items-center gap-3 justify-between">
        <div className="flex items-start gap-2 rounded-md bg-gray-50 border border-dashed border-gray-200 p-3 flex-1">
          <Info className="w-4 h-4 text-gray-500 mt-0.5 shrink-0" />
          <p className="text-[10px] text-gray-600 leading-relaxed">
            These settings drive the mock generators and monitoring views to
            illustrate how a real Connectors backend responds under different
            conditions. Higher frequencies and error rates surface more sync
            jobs, failures, and rate limit events to explore resilience and
            scaling patterns.
          </p>
        </div>
        <button
          type="button"
          onClick={handleResetDefaults}
          className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-md border border-gray-300 text-[10px] font-medium text-gray-700 bg-white hover:bg-gray-50 transition-colors"
        >
          <RefreshCw className="w-3 h-3" />
          Reset to defaults
        </button>
      </div>
    </section>
  );
}

/**
 * Small container for a labeled configuration field.
 */
function ConfigField({
  label,
  description,
  children,
}: {
  label: string;
  description?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="space-y-1.5">
      <div className="inline-flex items-center gap-1">
        <SlidersHorizontal className="w-3 h-3 text-gray-500" />
        <span className="text-xs font-semibold text-gray-800">{label}</span>
      </div>
      {description && (
        <p className="text-[10px] text-gray-500">{description}</p>
      )}
      <div>{children}</div>
    </div>
  );
}

/**
 * Generic pill-style selector row used for config values.
 */
function SelectPillRow({
  value,
  onChange,
  options,
}: {
  value: string;
  onChange: (next: string) => void;
  options: Array<{ value: string; label: string }>;
}) {
  return (
    <div className="inline-flex flex-wrap gap-1.5">
      {options.map((opt) => {
        const isActive = opt.value === value;
        return (
          <button
            key={opt.value}
            type="button"
            onClick={() => onChange(opt.value)}
            className={`px-2.5 py-1 rounded-full text-[10px] font-medium border transition-colors ${
              isActive
                ? "bg-gray-900 text-white border-gray-900"
                : "bg-white text-gray-700 border-gray-200 hover:bg-gray-50"
            }`}
          >
            {opt.label}
          </button>
        );
      })}
    </div>
  );
}

/**
 * Type guard to validate a stored DemoConfig shape before using it.
 */
function isValidConfig(value: unknown): value is DemoConfig {
  if (!value || typeof value !== "object") return false;
  const cfg = value as DemoConfig;
  const freqOk = ["low", "medium", "high"].includes(cfg.signalFrequency);
  const errOk = ["0%", "10%", "20%"].includes(cfg.errorRate);
  const timingOk = ["fast", "realistic"].includes(cfg.timingMode);
  const complexityOk = ["simple", "detailed"].includes(cfg.providerComplexity);
  const modeOk = ["mock", "real"].includes(cfg.mode);
  const validOk = typeof cfg.isConfigValid === "boolean";
  const errorsOk = Array.isArray(cfg.configErrors);
  const warningsOk = Array.isArray(cfg.configWarnings);
  return freqOk && errOk && timingOk && complexityOk && modeOk && validOk && errorsOk && warningsOk;
}

/**
 * Try to infer which preset (if any) matches a given config.
 */
function inferPresetKey(config: DemoConfig | null | undefined): string | null {
  if (!config) return null;
  for (const [key, preset] of Object.entries(PRESETS)) {
    const p = preset.config;
    if (
      p.signalFrequency === config.signalFrequency &&
      p.errorRate === config.errorRate &&
      p.timingMode === config.timingMode &&
      p.providerComplexity === config.providerComplexity
    ) {
      return key;
    }
  }
  return null;
}
