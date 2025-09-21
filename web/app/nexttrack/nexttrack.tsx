"use client";

import { useState, useRef } from "react";
import type { Track, Preferences, RecommendationRequest } from "./types";
import { presets } from "./constants/presets";
import { useRecommendationStream } from "./hooks/useRecommendationStream";
import Header from "./components/Header";
import TrackInput from "./components/TrackInput";
import AdvancedFilters from "./components/AdvancedFilters";
import PreferencesPanel from "./components/PreferencesPanel";
import RecommendationButton from "./components/RecommendationButton";
import PresetButtons from "./components/PresetButtons";
import RecommendationResult from "./components/RecommendationResult";

const NextTrack = () => {
  const [tracks, setTracks] = useState<Track[]>([{ id: 0, name: "" }]);
  const nextIdRef = useRef(1); // Keep track of the next available ID
  const [preferences, setPreferences] = useState<Preferences>({
    mood: 0.5,
    energy: 0.5,
    obscurity: 0.5,
    tempoVariance: 0.3,
    lyricalCoherence: 0.5,
    artistDiversity: true,
  });
  const [showRecommendation, setShowRecommendation] = useState(false);
  
  // Use the streaming hook
  const {
    status,
    candidates,
    recommendations,
    error,
    isStreaming,
    debugInfo,
    stats,
    streamRecommendations
  } = useRecommendationStream();

  const addTrack = () => {
    const newTrack = { id: nextIdRef.current, name: "" };
    nextIdRef.current += 1; // Increment for next time
    setTracks([...tracks, newTrack]);
  };

  const removeTrack = (id: number) => {
    setTracks(tracks.filter((track) => track.id !== id));
  };

  const updateTrack = (id: number, name: string) => {
    setTracks(
      tracks.map((track) => (track.id === id ? { ...track, name } : track))
    );
  };

  const handlePreferenceChange = (
    key: keyof Preferences,
    value: number | boolean
  ) => {
    setPreferences((prev) => ({ ...prev, [key]: value }));
  };

  const applyPreset = (presetName: string) => {
    const preset = presets[presetName];
    if (preset) {
      setPreferences((prev) => ({
        ...prev,
        mood: preset.mood,
        energy: preset.energy,
        obscurity: preset.obscurity,
        tempoVariance: preset.tempo,
        lyricalCoherence: preset.lyrical,
      }));
    }
  };

  const handleGetRecommendation = async () => {
    const validTracks = tracks.filter((track) => track.name.trim().length > 0);

    if (validTracks.length === 0) {
      alert("Please enter at least one track!");
      return;
    }

    setShowRecommendation(true);

    // Prepare the request
    const request: RecommendationRequest = {
      tracks: validTracks.map(track => track.name),
      preferences: {
        energy: preferences.energy,
        obscurity: preferences.obscurity,
        mood: preferences.mood,
      }
    };

    // Start streaming recommendations
    await streamRecommendations(request);
  };

  return (
    <div className="bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-100 min-h-screen">
      <Header />

      <main className="max-w-6xl mx-auto px-4 py-8">
        <div className="grid lg:grid-cols-3 gap-8">
          {/* Left Column: Track Input */}
          <div className="lg:col-span-2 space-y-6">
            <TrackInput
              tracks={tracks}
              onAddTrack={addTrack}
              onRemoveTrack={removeTrack}
              onUpdateTrack={updateTrack}
            />
            <AdvancedFilters />
          </div>

          {/* Right Column: Preferences */}
          <div className="space-y-6">
            <PreferencesPanel
              preferences={preferences}
              onPreferenceChange={handlePreferenceChange}
            />
            <RecommendationButton
              isLoading={isStreaming}
              onGetRecommendation={handleGetRecommendation}
            />
            <PresetButtons onApplyPreset={applyPreset} />
          </div>
        </div>

        <RecommendationResult 
          showRecommendation={showRecommendation}
          status={status}
          candidates={candidates}
          recommendations={recommendations}
          error={error}
          isStreaming={isStreaming}
          debugInfo={debugInfo}
          stats={stats}
        />
      </main>
    </div>
  );
};

export default NextTrack;
