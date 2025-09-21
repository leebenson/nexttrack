interface RecommendationButtonProps {
  isLoading: boolean;
  onGetRecommendation: () => void;
}

const RecommendationButton = ({
  isLoading,
  onGetRecommendation,
}: RecommendationButtonProps) => {
  return (
    <button
      onClick={onGetRecommendation}
      disabled={isLoading}
      className="w-full bg-gradient-to-r from-primary-600 to-accent-500 text-white font-semibold py-4 px-6 rounded-lg transition-all duration-200 transform hover:scale-[1.02] shadow-lg hover:shadow-xl disabled:opacity-50 disabled:cursor-not-allowed disabled:transform-none"
    >
      <div className="flex items-center justify-center space-x-2">
        {isLoading ? (
          <>
            <svg
              className="animate-spin w-5 h-5"
              fill="none"
              viewBox="0 0 24 24"
            >
              <circle
                className="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                strokeWidth="4"
              ></circle>
              <path
                className="opacity-75"
                fill="currentColor"
                d="m4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
              ></path>
            </svg>
            <span>Finding your next track...</span>
          </>
        ) : (
          <>
            <svg
              className="w-5 h-5"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                d="M13 10V3L4 14h7v7l9-11h-7z"
              ></path>
            </svg>
            <span>Get Recommendation</span>
          </>
        )}
      </div>
    </button>
  );
};

export default RecommendationButton;
