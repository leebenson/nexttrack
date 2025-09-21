interface AdvancedFiltersProps {}

const AdvancedFilters = () => {
  return (
    <div className="bg-white/70 backdrop-blur-sm rounded-lg border border-slate-200 p-6">
      <h3 className="text-lg font-semibold text-slate-900 mb-4">
        Advanced Filters
      </h3>
      <div className="grid md:grid-cols-2 gap-4">
        <div>
          <label className="block text-sm font-medium text-slate-700 mb-2">
            Year Range
          </label>
          <div className="flex space-x-2">
            <input
              type="number"
              placeholder="1960"
              className="w-full px-3 py-2 border border-slate-300 rounded-lg text-sm bg-white focus:ring-2 focus:ring-primary-500 focus:border-transparent"
            />
            <span className="text-slate-500 self-center">to</span>
            <input
              type="number"
              placeholder="2024"
              className="w-full px-3 py-2 border border-slate-300 rounded-lg text-sm bg-white focus:ring-2 focus:ring-primary-500 focus:border-transparent"
            />
          </div>
        </div>
        <div>
          <label className="block text-sm font-medium text-slate-700 mb-2">
            Exclude Artists
          </label>
          <input
            type="text"
            placeholder="Artist names (comma separated)"
            className="w-full px-3 py-2 border border-slate-300 rounded-lg text-sm bg-white focus:ring-2 focus:ring-primary-500 focus:border-transparent"
          />
        </div>
      </div>
      <div className="mt-4 flex items-center">
        <input
          type="checkbox"
          id="exclude-explicit"
          className="w-4 h-4 text-primary-600 border-slate-300 rounded focus:ring-primary-500"
        />
        <label
          htmlFor="exclude-explicit"
          className="ml-2 text-sm text-slate-700"
        >
          Exclude explicit content
        </label>
      </div>
    </div>
  );
};

export default AdvancedFilters;
