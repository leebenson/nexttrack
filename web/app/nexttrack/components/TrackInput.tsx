import type { Track } from "../types";

interface TrackInputProps {
  tracks: Track[];
  onAddTrack: () => void;
  onRemoveTrack: (id: number) => void;
  onUpdateTrack: (id: number, name: string) => void;
}

const TrackInput = ({
  tracks,
  onAddTrack,
  onRemoveTrack,
  onUpdateTrack,
}: TrackInputProps) => {
  return (
    <div className="gradient-border">
      <div className="gradient-border-inner p-6">
        <h2 className="text-xl font-semibold text-slate-900 mb-4 flex items-center">
          <svg
            className="w-5 h-5 mr-2 text-primary-600"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="2"
              d="M12 4v16m8-8H4"
            ></path>
          </svg>
          Your Track Sequence
        </h2>
        <p className="description-text text-slate-600 mb-4">
          Enter the songs you've been listening to. NextTrack will recommend
          what to play next based on this sequence.
        </p>
        <div className="mb-6 p-3 bg-blue-50 border border-blue-200 rounded-lg">
          <p className="text-xs text-blue-700 font-medium mb-1">Format tips:</p>
          <ul className="text-xs text-blue-600 space-y-1">
            <li>• "Song Title - Artist Name" (e.g., All the Small Things - Blink-182)</li>
            <li>• "Song Title by Artist Name" (e.g., Bohemian Rhapsody by Queen)</li>
            <li>• Best results with full artist names including numbers/symbols</li>
          </ul>
        </div>

        <div className="space-y-3 mb-4">
          {tracks.map((track, index) => (
            <div
              key={track.id}
              className="track-input-group flex items-center space-x-3 p-3 bg-slate-50 rounded-lg border border-slate-200"
            >
              <div className="w-8 h-8 bg-primary-100 text-primary-600 rounded-full flex items-center justify-center text-sm font-medium">
                {index + 1}
              </div>
              <input
                type="text"
                placeholder="Track Name - Artist (e.g., Bohemian Rhapsody - Queen)"
                className="flex-1 px-3 py-2 border border-slate-300 rounded-lg text-sm bg-white focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                value={track.name}
                onChange={(e) => onUpdateTrack(track.id, e.target.value)}
              />
              {index > 0 && (
                <button
                  onClick={() => onRemoveTrack(track.id)}
                  className="remove-track w-8 h-8 text-slate-400 hover:text-red-500 transition-colors"
                >
                  <svg
                    className="w-4 h-4"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth="2"
                      d="M6 18L18 6M6 6l12 12"
                    ></path>
                  </svg>
                </button>
              )}
            </div>
          ))}
        </div>

        <div
          onClick={onAddTrack}
          className="text-primary-600 underline font-medium cursor-pointer flex items-center space-x-2"
          style={{ width: "fit-content" }}
        >
          <svg
            className="w-4 h-4"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="2"
              d="M12 4v16m8-8H4"
            ></path>
          </svg>
          <span>Add another track</span>
        </div>
      </div>
    </div>
  );
};

export default TrackInput;
