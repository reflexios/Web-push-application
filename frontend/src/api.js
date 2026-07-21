const BASE_URL = '/api';

export class ApiError extends Error {
  constructor(message, status) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
  }
}

async function request(path, options = {}) {
  const res = await fetch(`${BASE_URL}${path}`, {
    credentials: 'include',
    headers: {
      'Content-Type': 'application/json',
      ...options.headers,
    },
    ...options,
  });

  if (res.status === 204) {
    return null;
  }

  let body = null;
  const text = await res.text();
  if (text) {
    try {
      body = JSON.parse(text);
    } catch {
      body = text;
    }
  }

  if (!res.ok) {
    const message =
      (body && typeof body === 'object' && body.error) ||
      (typeof body === 'string' && body) ||
      `Request error (${res.status})`;
    throw new ApiError(message, res.status);
  }

  return body;
}

export const api = {
  login(login, password) {
    return request('/login', {
      method: 'POST',
      body: JSON.stringify({ login, password }),
    });
  },

  logout() {
    return request('/logout', { method: 'POST' });
  },

  listClients({ page = 1, perPage = 20, search = '' } = {}) {
    const params = new URLSearchParams({
      page: String(page),
      per_page: String(perPage),
    });
    if (search.trim()) params.set('search', search.trim());
    return request(`/clients?${params.toString()}`);
  },

  getClient(clientId) {
    return request(`/clients/${clientId}`);
  },

  createClient({ name, vapidSubject }) {
    return request('/clients', {
      method: 'POST',
      body: JSON.stringify({
        name,
        vapid_subject: vapidSubject || undefined,
      }),
    });
  },

  deleteClient(clientId) {
    return request(`/clients/${clientId}`, { method: 'DELETE' });
  },

  regenerateApiKey(clientId) {
    return request(`/clients/${clientId}/regenerate-api-key`, { method: 'POST' });
  },

  listSubscriptions(clientId, { page = 1, perPage = 20 } = {}) {
    const params = new URLSearchParams({
      page: String(page),
      per_page: String(perPage),
    });
    return request(`/clients/${clientId}/subscriptions?${params.toString()}`);
  },
};
