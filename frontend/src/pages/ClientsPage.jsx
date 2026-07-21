import { useCallback, useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { api } from '../api';
import { Pagination } from '../components/Pagination';
import { CreateClientModal } from '../components/CreateClientModal';
import { RegenerateApiKeyModal } from '../components/RegenerateApiKeyModal';
import { ConfirmModal } from '../components/ConfirmModal';
import { CopyableValue } from '../components/CopyableValue';

const DEFAULT_PER_PAGE = 10;

export function ClientsPage() {
  const navigate = useNavigate();
  const [data, setData] = useState({ items: [], page: 1, per_page: DEFAULT_PER_PAGE, total: 0, total_pages: 0 });
  const [page, setPage] = useState(1);
  const [perPage, setPerPage] = useState(DEFAULT_PER_PAGE);
  const [search, setSearch] = useState('');
  const [debouncedSearch, setDebouncedSearch] = useState('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [showCreate, setShowCreate] = useState(false);
  const [clientToDelete, setClientToDelete] = useState(null);
  const [clientToRegenerate, setClientToRegenerate] = useState(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError('');
    try {
      const result = await api.listClients({ page, perPage, search: debouncedSearch });
      setData(result);
    } catch (err) {
      setError(err.message || 'Failed to load the client list');
    } finally {
      setLoading(false);
    }
  }, [page, perPage, debouncedSearch]);

  useEffect(() => {
    const timer = setTimeout(() => {
      setPage(1);
      setDebouncedSearch(search);
    }, 400);
    return () => clearTimeout(timer);
  }, [search]);

  useEffect(() => {
    load();
  }, [load]);

  function handleSearchSubmit(e) {
    e.preventDefault();
    setPage(1);
    setDebouncedSearch(search);
  }

  function handlePerPageChange(size) {
    setPerPage(size);
    setPage(1);
  }

  async function handleDelete() {
    await api.deleteClient(clientToDelete.client_id);
    setClientToDelete(null);
    if (data.items.length === 1 && page > 1) {
      setPage((p) => p - 1);
    } else {
      load();
    }
  }

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <h1>Clients</h1>
          <p className="page-subtitle">Projects that send push notifications through the platform</p>
        </div>
        <button className="btn btn-primary" onClick={() => setShowCreate(true)}>
          + Add client
        </button>
      </div>

      <form onSubmit={handleSearchSubmit} style={{ marginBottom: 16, maxWidth: 320 }}>
        <input
          className="input"
          placeholder="Search by name…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
      </form>

      {error && <div className="alert alert-error">{error}</div>}

      <div className="surface">
        {loading ? (
          <div className="loading-state">
            <span className="spinner" />
            Loading…
          </div>
        ) : data.items.length === 0 ? (
          <div className="empty-state">
            <div className="empty-state-title">No clients yet</div>
            <p>Add your first client to get an API key and a VAPID key pair.</p>
          </div>
        ) : (
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>Name</th>
                  <th>VAPID subject</th>
                  <th>Created</th>
                  <th>Client ID</th>
                  <th>VAPID public key</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {data.items.map((client) => (
                  <tr key={client.client_id}>
                    <td className="client-name">{client.name}</td>
                    <td className="text-muted">{client.vapid_subject}</td>
                    <td className="text-muted">
                      {new Date(client.created_at).toLocaleDateString('en-US', {
                        day: '2-digit',
                        month: '2-digit',
                        year: 'numeric',
                      })}
                    </td>
                    <td>
                      <CopyableValue value={client.client_id} />
                    </td>
                    <td>
                      <CopyableValue value={client.vapid_public_key} keep={8} />
                    </td>
                    <td>
                      <div className="cell-actions">
                        <button
                          className="btn btn-secondary btn-sm"
                          onClick={() => navigate(`/clients/${client.client_id}/subscriptions`)}
                        >
                          Subscriptions
                        </button>
                        <button
                          className="icon-btn"
                          title="Regenerate API key"
                          onClick={() => setClientToRegenerate(client)}
                        >
                          ⟳
                        </button>
                        <button
                          className="icon-btn danger"
                          title="Delete client"
                          onClick={() => setClientToDelete(client)}
                        >
                          ✕
                        </button>
                      </div>
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

      {showCreate && (
        <CreateClientModal
          onClose={() => setShowCreate(false)}
          onCreated={() => {
            setPage(1);
            load();
          }}
        />
      )}

      {clientToRegenerate && (
        <RegenerateApiKeyModal
          client={clientToRegenerate}
          onClose={() => setClientToRegenerate(null)}
        />
      )}

      {clientToDelete && (
        <ConfirmModal
          title="Delete client?"
          description={`Client "${clientToDelete.name}" and all of its subscriptions will be permanently deleted.`}
          onConfirm={handleDelete}
          onClose={() => setClientToDelete(null)}
        />
      )}
    </div>
  );
}
