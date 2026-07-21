import { useCallback, useEffect, useState } from 'react';
import { Link, useParams } from 'react-router-dom';
import { api } from '../api';
import { Pagination } from '../components/Pagination';

const DEFAULT_PER_PAGE = 10;

function truncate(str, max = 46) {
  if (!str) return '';
  return str.length > max ? `${str.slice(0, max)}…` : str;
}

export function ClientSubscriptionsPage() {
  const { clientId } = useParams();
  const [client, setClient] = useState(null);
  const [data, setData] = useState({ items: [], page: 1, per_page: DEFAULT_PER_PAGE, total: 0, total_pages: 0 });
  const [page, setPage] = useState(1);
  const [perPage, setPerPage] = useState(DEFAULT_PER_PAGE);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  const load = useCallback(async () => {
    setLoading(true);
    setError('');
    try {
      const [clientInfo, subs] = await Promise.all([
        client ? Promise.resolve(client) : api.getClient(clientId),
        api.listSubscriptions(clientId, { page, perPage }),
      ]);
      setClient(clientInfo);
      setData(subs);
    } catch (err) {
      setError(err.message || 'Failed to load subscriptions');
    } finally {
      setLoading(false);
    }
  }, [clientId, page, perPage]);

  useEffect(() => {
    load();
  }, [clientId, page, perPage]);

  function handlePerPageChange(size) {
    setPerPage(size);
    setPage(1);
  }

  return (
    <div className="page">
      <div className="breadcrumb">
        <Link to="/clients">← Clients</Link>
      </div>

      <div className="page-header">
        <div>
          <h1>{client ? client.name : 'Subscriptions'}</h1>
          <p className="page-subtitle">
            All active push subscriptions for this client
            {data.total > 0 && <> · <span className="badge">{data.total}</span></>}
          </p>
        </div>
      </div>

      {error && <div className="alert alert-error">{error}</div>}

      <div className="surface">
        {loading ? (
          <div className="loading-state">
            <span className="spinner" />
            Loading…
          </div>
        ) : data.items.length === 0 ? (
          <div className="empty-state">
            <div className="empty-state-title">No subscriptions yet</div>
            <p>Once users subscribe to push, they will show up here.</p>
          </div>
        ) : (
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>Endpoint</th>
                  <th>User agent</th>
                  <th>Created</th>
                  <th>Updated</th>
                </tr>
              </thead>
              <tbody>
                {data.items.map((sub) => (
                  <tr key={sub.id}>
                    <td className="mono" title={sub.endpoint}>
                      {truncate(sub.endpoint)}
                    </td>
                    <td className="text-muted" title={sub.user_agent || ''}>
                      {truncate(sub.user_agent, 36) || '-'}
                    </td>
                    <td className="text-muted">
                      {new Date(sub.created_at).toLocaleDateString('en-US')}
                    </td>
                    <td className="text-muted">
                      {new Date(sub.updated_at).toLocaleDateString('en-US')}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

        {!loading && data.items.length > 0 && (
          <Pagination
            page={data.page}
            totalPages={data.total_pages}
            total={data.total}
            perPage={data.per_page}
            onPageChange={setPage}
            onPerPageChange={handlePerPageChange}
          />
        )}
      </div>
    </div>
  );
}
