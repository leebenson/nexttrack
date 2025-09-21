import { motion, AnimatePresence } from "framer-motion";
import { useState } from "react";
import type { ApiTrack } from "../types";

interface RecommendationResultProps {
  showRecommendation: boolean;
  status: string;
  candidates: Array<{ track: ApiTrack; score: number }>;
  recommendations: ApiTrack[];
  error: string | null;
  isStreaming: boolean;
  debugInfo: string[];
  stats: {
    totalCandidatesFound: number;
    filteredByObscurity: number;
    notFoundInMusicBrainz: number;
  };
}

const RecommendationResult = ({
  showRecommendation,
  status,
  candidates,
  recommendations,
  error,
  isStreaming,
  debugInfo,
  stats,
}: RecommendationResultProps) => {
  if (!showRecommendation) return null;

  // Show error state
  if (error) {
    return (
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        className="mt-8"
      >
        <div className="bg-red-50 border border-red-200 rounded-lg p-6">
          <h2 className="text-xl font-semibold text-red-900 mb-2">Error</h2>
          <p className="text-red-700">{error}</p>
        </div>
      </motion.div>
    );
  }

  // Show streaming progress
  if (isStreaming || (recommendations.length === 0 && !error)) {
    return (
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        className="mt-8"
      >
        <div className="bg-white/80 backdrop-blur-sm rounded-lg border border-slate-200 p-6">
          <h2 className="text-xl font-semibold text-slate-900 mb-4">
            Finding Your Next Track...
          </h2>
          
          {/* Status message */}
          <div className="mb-4">
            <div className="flex items-center justify-between mb-2">
              <p className="text-sm font-medium text-slate-700">{status}</p>
              {candidates.length > 0 && (
                <motion.div
                  initial={{ opacity: 0, scale: 0.8 }}
                  animate={{ opacity: 1, scale: 1 }}
                  className="flex items-center space-x-2"
                >
                  <div className="flex space-x-1">
                    {[...Array(3)].map((_, i) => (
                      <motion.div
                        key={i}
                        className="w-1.5 h-1.5 bg-blue-500 rounded-full"
                        animate={{ y: [-2, 2, -2] }}
                        transition={{
                          duration: 0.6,
                          repeat: Infinity,
                          delay: i * 0.2,
                        }}
                      />
                    ))}
                  </div>
                  <span className="text-xs text-slate-500">
                    Processing
                  </span>
                </motion.div>
              )}
            </div>
            <div className="relative h-2 bg-slate-200 rounded-full overflow-hidden">
              <motion.div
                className="absolute inset-0 h-full bg-gradient-to-r from-blue-500 via-indigo-500 to-purple-500"
                initial={{ x: "-100%" }}
                animate={{ x: "100%" }}
                transition={{ duration: 2, repeat: Infinity, ease: "linear" }}
              />
              <div className="absolute inset-0 h-full bg-white/20 backdrop-blur-sm" />
            </div>
          </div>

          {/* Real-time stats */}
          {(stats.totalCandidatesFound > 0 || candidates.length > 0) && (
            <motion.div
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.3 }}
              className="mt-4 grid grid-cols-3 gap-3"
            >
              <div className="text-center p-2 bg-blue-50 rounded-lg">
                <p className="text-2xl font-bold text-blue-600">
                  {candidates.length}
                </p>
                <p className="text-xs text-blue-700">Analyzed</p>
              </div>
              <div className="text-center p-2 bg-green-50 rounded-lg">
                <p className="text-2xl font-bold text-green-600">
                  {stats.totalCandidatesFound}
                </p>
                <p className="text-xs text-green-700">Found</p>
              </div>
              <div className="text-center p-2 bg-purple-50 rounded-lg">
                <p className="text-2xl font-bold text-purple-600">
                  {candidates.filter(c => c.score >= 0.6).length}
                </p>
                <p className="text-xs text-purple-700">Good Matches</p>
              </div>
            </motion.div>
          )}

          {/* Live candidates */}
          {candidates.length > 0 && (
            <div className="mt-6">
              <h3 className="text-sm font-medium text-slate-700 mb-3">
                Analyzing candidates ({candidates.length} found)
              </h3>
              <div className="space-y-1.5 max-h-96 overflow-y-auto pr-2">
                <AnimatePresence mode="popLayout">
                  {candidates.slice(-15).reverse().map((candidate, idx) => (
                    <motion.div
                      key={`${candidate.track.id}-${candidate.track.artist}-${idx}`}
                      initial={{ opacity: 0, x: -20, height: 0 }}
                      animate={{ opacity: 1, x: 0, height: "auto" }}
                      exit={{ opacity: 0, x: 20, height: 0 }}
                      transition={{ 
                        type: "spring",
                        stiffness: 500,
                        damping: 30,
                        opacity: { duration: 0.2 }
                      }}
                      className="flex items-center justify-between p-2.5 bg-gradient-to-r from-slate-50 to-slate-100 rounded-lg border border-slate-200/50 hover:border-slate-300 transition-colors"
                    >
                      <div className="flex-1 min-w-0">
                        <p className="text-sm font-medium text-slate-900 truncate">
                          {candidate.track.name}
                        </p>
                        <p className="text-xs text-slate-600 truncate">
                          {candidate.track.artist}
                        </p>
                      </div>
                      <div className="ml-3 text-right">
                        <span className={`text-sm font-medium ${
                          candidate.score >= 0.8 ? 'text-green-600' : 
                          candidate.score >= 0.6 ? 'text-blue-600' : 
                          candidate.score >= 0.4 ? 'text-amber-600' : 'text-slate-600'
                        }`}>
                          {(candidate.score * 100).toFixed(0)}%
                        </span>
                        {candidate.track.popularity > 0 && (
                          <p className="text-xs text-slate-500">
                            Pop: {candidate.track.popularity}%
                          </p>
                        )}
                      </div>
                    </motion.div>
                  ))}
                </AnimatePresence>
              </div>
              {candidates.length > 15 && (
                <p className="text-xs text-slate-500 mt-2 text-center">
                  Showing last 15 of {candidates.length} candidates
                </p>
              )}
            </div>
          )}

          {/* Debug info */}
          {debugInfo.length > 0 && (
            <div className="mt-6">
              <details className="text-xs">
                <summary className="cursor-pointer text-slate-500 hover:text-slate-700">
                  Show search details ({debugInfo.length} entries)
                </summary>
                <div className="mt-2 max-h-32 overflow-y-auto bg-slate-50 rounded p-2">
                  {debugInfo.map((info, idx) => (
                    <div key={idx} className="text-slate-600 font-mono">
                      {info}
                    </div>
                  ))}
                </div>
              </details>
            </div>
          )}
        </div>
      </motion.div>
    );
  }

  // Show final recommendations
  return (
    <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        className="mt-8"
      >
      <div className="bg-white/80 backdrop-blur-sm rounded-lg border border-slate-200 p-6">
        <h2 className="text-xl font-semibold text-slate-900 mb-4">
          Your Next Tracks
        </h2>
        
        <div className="space-y-4">
          {recommendations.map((track, idx) => (
            <motion.div
              key={track.id}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: idx * 0.1 }}
              className="border-b border-slate-100 last:border-0 pb-4 last:pb-0"
            >
              <div className="flex items-start space-x-4">
                <div className="relative w-20 h-20 rounded-lg overflow-hidden">
                  {track.album_art ? (
                    <img 
                      src={track.album_art} 
                      alt={`${track.name} album cover`}
                      className="w-full h-full object-cover"
                      onError={(e) => {
                        // Hide the broken image
                        (e.target as HTMLImageElement).style.display = 'none';
                      }}
                    />
                  ) : (
                    <div className="w-full h-full bg-gradient-to-br from-slate-200 to-slate-300 flex items-center justify-center">
                      <svg className="w-10 h-10 text-slate-400" fill="currentColor" viewBox="0 0 24 24">
                        <path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"/>
                      </svg>
                    </div>
                  )}
                  <div className="absolute top-1 left-1 w-6 h-6 bg-white/90 rounded-full flex items-center justify-center text-sm font-bold text-slate-700">
                    {idx + 1}
                  </div>
                </div>
                <div className="flex-1">
                  <h3 className="font-semibold text-lg text-slate-900">
                    {track.name}
                  </h3>
                  <p className="text-slate-600 mb-2">{track.artist}</p>
                  <div className="flex flex-wrap gap-2">
                    {track.popularity > 70 && (
                      <span className="px-2 py-1 bg-green-100 text-green-700 text-xs rounded-full">
                        Popular
                      </span>
                    )}
                    {track.features.energy > 0.7 && (
                      <span className="px-2 py-1 bg-orange-100 text-orange-700 text-xs rounded-full">
                        High Energy
                      </span>
                    )}
                    {track.features.valence > 0.7 && (
                      <span className="px-2 py-1 bg-yellow-100 text-yellow-700 text-xs rounded-full">
                        Upbeat
                      </span>
                    )}
                  </div>
                </div>
              </div>
            </motion.div>
          ))}
        </div>
      </div>
    </motion.div>
  );
};

export default RecommendationResult;
