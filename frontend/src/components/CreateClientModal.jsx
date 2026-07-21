import { useState } from 'react';
import { Modal } from './Modal';
import { ApiKeyReveal } from './ApiKeyReveal';
import { api } from '../api';

export function CreateClientModal({ onClose, onCreated }) {
  const [name, setName] = useState('');
  const [vapidSubject, setVapidSubject] = useState('');
  const [error, setError] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [created, setCreated] = useState(null);

  async function handleSubmit(e) {
    e.preventDefault();
    if (!name.trim()) {
      setError('Please enter a client name');
      return;
    }
    setError('');
    setSubmitting(true);
    try {
      const result = await api.createClient({ name: name.trim(), vapidSubject: vapidSubject.trim() });
      setCreated(result);
      onCreated?.();
    } catch (err) {
      setError(err.message || 'Failed to create client');
    } finally {
      setSubmitting(false);
    }
  }

  if (created) {
    return (
      <Modal title="Client created" onClose={onClose}>
        <div className="alert alert-success">"{created.name}" was successfully registered</div>
        <ApiKeyReveal result={created} onDone={onClose} />
      </Modal>
    );
  }

  return (
    <Modal title="New client" onClose={onClose}>
      <form onSubmit={handleSubmit}>
        {error && <div className="alert alert-error">{error}</div>}

        <div className="field">
          <label htmlFor="client-name">Name</label>
          <input
            id="client-name"
            className="input"
            placeholder="e.g. my-shop.com"
            value={name}
            onChange={(e) => setName(e.target.value)}
            autoFocus
            required
          />
        </div>

        <div className="field">
          <label htmlFor="vapid-subject">VAPID subject (optional)</label>
          <input
            id="vapid-subject"
            className="input"
            placeholder="mailto:admin@example.com"
            value={vapidSubject}
            onChange={(e) => setVapidSubject(e.target.value)}
          />
          <p className="hint">If left blank, the default address will be used.</p>
        </div>

        <div className="modal-footer">
          <button type="button" className="btn btn-secondary" onClick={onClose}>
            Cancel
          </button>
          <button type="submit" className="btn btn-primary" disabled={submitting}>
            {submitting ? 'Creating…' : 'Create'}
          </button>
        </div>
      </form>
    </Modal>
  );
}
