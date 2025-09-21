import { presets } from "../constants/presets";

interface PresetButtonsProps {
  onApplyPreset: (presetName: string) => void;
}

const PresetButtons = ({ onApplyPreset }: PresetButtonsProps) => {
  return (
    <div className="space-y-2">
      <h3 className="text-sm font-medium text-slate-700">Quick Presets</h3>
      <div className="grid grid-cols-2 gap-2">
        {Object.entries(presets).map(([name, _]) => (
          <button
            key={name}
            onClick={() => onApplyPreset(name)}
            className="preset-btn px-3 py-2 text-xs bg-slate-100 hover:bg-slate-200 rounded-lg transition-colors"
          >
            {name === "discovery" && "🔍 Discovery Mode"}
            {name === "chill" && "😌 Chill Vibes"}
            {name === "energy" && "⚡ High Energy"}
            {name === "hits" && "🎵 Popular Hits"}
          </button>
        ))}
      </div>
    </div>
  );
};

export default PresetButtons;
