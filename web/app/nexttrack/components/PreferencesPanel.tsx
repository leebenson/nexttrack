import type { Preferences } from "../types";

interface PreferencesPanelProps {
  preferences: Preferences;
  onPreferenceChange: (key: keyof Preferences, value: number | boolean) => void;
}

const PreferencesPanel = ({
  preferences,
  onPreferenceChange,
}: PreferencesPanelProps) => {
  return (
    <div className="bg-white/70 backdrop-blur-sm rounded-lg border border-slate-200 p-6">
      <h2 className="text-xl font-semibold text-slate-900 mb-6 flex items-center">
        <svg
          className="w-5 h-5 mr-2 text-accent-500"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth="2"
            d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 100 4m0-4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 100 4m0-4v2m0-6V4"
          ></path>
        </svg>
        Preferences
      </h2>

      <div className="space-y-6">
        {/* Mood Slider */}
        <div>
          <div className="flex justify-between items-center mb-2">
            <label className="text-sm font-medium text-slate-700">Mood</label>
            <span className="text-sm text-slate-500">
              {preferences.mood.toFixed(1)}
            </span>
          </div>
          <input
            type="range"
            min="0"
            max="1"
            step="0.1"
            value={preferences.mood}
            onChange={(e) =>
              onPreferenceChange("mood", parseFloat(e.target.value))
            }
            className="w-full h-2 bg-slate-200 rounded-lg appearance-none cursor-pointer accent-primary-500"
          />
          <div className="flex justify-between text-xs text-slate-500 mt-1">
            <span>Melancholy</span>
            <span>Uplifting</span>
          </div>
        </div>

        {/* Energy Slider */}
        <div>
          <div className="flex justify-between items-center mb-2">
            <label className="text-sm font-medium text-slate-700">Energy</label>
            <span className="text-sm text-slate-500">
              {preferences.energy.toFixed(1)}
            </span>
          </div>
          <input
            type="range"
            min="0"
            max="1"
            step="0.1"
            value={preferences.energy}
            onChange={(e) =>
              onPreferenceChange("energy", parseFloat(e.target.value))
            }
            className="w-full h-2 bg-slate-200 rounded-lg appearance-none cursor-pointer accent-primary-500"
          />
          <div className="flex justify-between text-xs text-slate-500 mt-1">
            <span>Chill</span>
            <span>High Energy</span>
          </div>
        </div>

        {/* Obscurity Slider */}
        <div>
          <div className="flex justify-between items-center mb-2">
            <label className="text-sm font-medium text-slate-700">
              Discovery Level
            </label>
            <span className="text-sm text-slate-500">
              {preferences.obscurity.toFixed(1)}
            </span>
          </div>
          <input
            type="range"
            min="0"
            max="1"
            step="0.1"
            value={preferences.obscurity}
            onChange={(e) =>
              onPreferenceChange("obscurity", parseFloat(e.target.value))
            }
            className="w-full h-2 bg-slate-200 rounded-lg appearance-none cursor-pointer accent-primary-500"
          />
          <div className="flex justify-between text-xs text-slate-500 mt-1">
            <span>Popular Hits</span>
            <span>Hidden Gems</span>
          </div>
          <p className="text-xs text-slate-400 mt-2">
            Low values (0.0-0.3) = mainstream hits, High values (0.7-1.0) = obscure tracks
          </p>
        </div>

        {/* Tempo Variance Slider */}
        <div>
          <div className="flex justify-between items-center mb-2">
            <label className="text-sm font-medium text-slate-700">
              Tempo Variance
            </label>
            <span className="text-sm text-slate-500">
              {preferences.tempoVariance.toFixed(1)}
            </span>
          </div>
          <input
            type="range"
            min="0"
            max="1"
            step="0.1"
            value={preferences.tempoVariance}
            onChange={(e) =>
              onPreferenceChange("tempoVariance", parseFloat(e.target.value))
            }
            className="w-full h-2 bg-slate-200 rounded-lg appearance-none cursor-pointer accent-primary-500"
          />
          <div className="flex justify-between text-xs text-slate-500 mt-1">
            <span>Similar Tempo</span>
            <span>Mix It Up</span>
          </div>
        </div>

        {/* Lyrical Coherence Slider */}
        <div>
          <div className="flex justify-between items-center mb-2">
            <label className="text-sm font-medium text-slate-700">
              Lyrical Coherence
            </label>
            <span className="text-sm text-slate-500">
              {preferences.lyricalCoherence.toFixed(1)}
            </span>
          </div>
          <input
            type="range"
            min="0"
            max="1"
            step="0.1"
            value={preferences.lyricalCoherence}
            onChange={(e) =>
              onPreferenceChange("lyricalCoherence", parseFloat(e.target.value))
            }
            className="w-full h-2 bg-slate-200 rounded-lg appearance-none cursor-pointer accent-primary-500"
          />
          <div className="flex justify-between text-xs text-slate-500 mt-1">
            <span>Theme Variety</span>
            <span>Consistent Theme</span>
          </div>
        </div>

        {/* Toggle Options */}
        <div className="space-y-3 pt-4 border-t border-slate-200">
          <div className="flex items-center justify-between">
            <label className="text-sm font-medium text-slate-700">
              Artist Diversity
            </label>
            <div className="relative">
              <input
                type="checkbox"
                id="artist-diversity"
                checked={preferences.artistDiversity}
                onChange={(e) =>
                  onPreferenceChange("artistDiversity", e.target.checked)
                }
                className="sr-only"
              />
              <div
                className={`toggle-bg w-10 h-6 rounded-full shadow-inner cursor-pointer ${
                  preferences.artistDiversity ? "on" : ""
                }`}
              ></div>
              <div
                className={`toggle-dot absolute w-4 h-4 bg-white rounded-full shadow left-1 top-1 ${
                  preferences.artistDiversity ? "on" : ""
                }`}
              ></div>
            </div>
          </div>
          <p className="text-xs text-slate-500">
            Avoid repeating the same artists
          </p>
        </div>
      </div>
    </div>
  );
};

export default PreferencesPanel;
