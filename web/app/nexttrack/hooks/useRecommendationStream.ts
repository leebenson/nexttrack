import { useState, useCallback } from 'react';
import type { ApiTrack, StreamEvent, RecommendationRequest } from '../types';

interface StreamState {
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

export const useRecommendationStream = () => {
  const [state, setState] = useState<StreamState>({
    status: '',
    candidates: [],
    recommendations: [],
    error: null,
    isStreaming: false,
    debugInfo: [],
    stats: {
      totalCandidatesFound: 0,
      filteredByObscurity: 0,
      notFoundInMusicBrainz: 0,
    },
  });

  const streamRecommendations = useCallback(async (request: RecommendationRequest) => {
    setState({
      status: 'Connecting...',
      candidates: [],
      recommendations: [],
      error: null,
      isStreaming: true,
      debugInfo: [],
      stats: {
        totalCandidatesFound: 0,
        filteredByObscurity: 0,
        notFoundInMusicBrainz: 0,
      },
    });

    try {
      const response = await fetch('http://localhost:3000/mb/recommend/stream', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(request),
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const reader = response.body?.getReader();
      const decoder = new TextDecoder();

      if (!reader) {
        throw new Error('No response body');
      }

      let buffer = '';

      while (true) {
        const { done, value } = await reader.read();
        
        if (done) break;

        buffer += decoder.decode(value, { stream: true });
        
        // Process complete SSE events
        const lines = buffer.split('\n');
        buffer = lines.pop() || '';

        for (const line of lines) {
          if (line.startsWith('data: ')) {
            try {
              const event = JSON.parse(line.slice(6)) as StreamEvent;
              
              switch (event.type) {
                case 'Status':
                  setState(prev => {
                    // Parse status messages for stats
                    const foundMatch = event.message.match(/Found (\d+) similar tracks to process/);
                    if (foundMatch) {
                      return {
                        ...prev,
                        status: event.message,
                        stats: { ...prev.stats, totalCandidatesFound: parseInt(foundMatch[1]) },
                      };
                    }
                    return {
                      ...prev,
                      status: event.message,
                    };
                  });
                  break;
                
                case 'Candidate':
                  setState(prev => ({
                    ...prev,
                    candidates: [...prev.candidates, { track: event.track, score: event.score }]
                      .sort((a, b) => b.score - a.score)
                      .slice(0, 20), // Keep top 20 candidates
                  }));
                  break;
                
                case 'Complete':
                  setState(prev => ({
                    ...prev,
                    recommendations: event.tracks,
                    isStreaming: false,
                    status: 'Complete!',
                  }));
                  break;
                
                case 'Error':
                  setState(prev => ({
                    ...prev,
                    error: event.message,
                    isStreaming: false,
                  }));
                  break;
                
                case 'Debug':
                  setState(prev => ({
                    ...prev,
                    debugInfo: [...prev.debugInfo, event.message],
                  }));
                  break;
              }
            } catch (e) {
              console.error('Failed to parse SSE event:', e);
            }
          }
        }
      }
    } catch (error) {
      setState(prev => ({
        ...prev,
        error: error instanceof Error ? error.message : 'Unknown error',
        isStreaming: false,
      }));
    }
  }, []);

  return { ...state, streamRecommendations };
};