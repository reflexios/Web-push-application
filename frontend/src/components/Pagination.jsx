const PAGE_SIZE_OPTIONS = [5, 10, 20, 50, 100];

export function Pagination({ page, totalPages, total, perPage, onPageChange, onPerPageChange }) {
  if (total === 0) return null;

  const from = (page - 1) * perPage + 1;
  const to = Math.min(page * perPage, total);

  return (
    <div className="pagination">
      <div className="pagination-info">
        {from}–{to} of {total}
      </div>

      {onPerPageChange && (
        <div className="pagination-page-size">
          <label htmlFor="per-page-select">Rows per page:</label>
          <select
            id="per-page-select"
            className="select"
            value={perPage}
            onChange={(e) => onPerPageChange(Number(e.target.value))}
          >
            {PAGE_SIZE_OPTIONS.map((size) => (
              <option key={size} value={size}>
                {size}
              </option>
            ))}
          </select>
        </div>
      )}

      <div className="pagination-controls">
        <button
          className="btn btn-secondary btn-sm"
          disabled={page <= 1}
          onClick={() => onPageChange(page - 1)}
        >
          Back
        </button>
        <span className="pagination-page">
          page {page} of {Math.max(totalPages, 1)}
        </span>
        <button
          className="btn btn-secondary btn-sm"
          disabled={page >= totalPages}
          onClick={() => onPageChange(page + 1)}
        >
          Next
        </button>
      </div>
    </div>
  );
}
