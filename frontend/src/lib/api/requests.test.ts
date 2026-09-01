import { describe, it, expect, vi, beforeEach } from 'vitest';

// Test the Fic Requests API client by mocking fetch.
const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
});

async function importRequests() {
  return await import('./requests');
}

function okJson(body: unknown) {
  return { ok: true, json: async () => body };
}

describe('listRequests', () => {
  it('GETs /api/requests with status/sort/page and returns items', async () => {
    mockFetch.mockResolvedValue(
      okJson({ err: 0, items: [{ id: 1, title: 'A prompt', body: 'b', seed_work_id: null, status: 'open', created_at: 'c', answer_count: 0 }] }),
    );
    const { listRequests } = await importRequests();
    const res = await listRequests('open', 'new', 2);
    expect(mockFetch.mock.calls[0][0]).toBe('/api/requests?status=open&sort=new&page=2');
    expect(res.items).toHaveLength(1);
  });
});

describe('getRequest', () => {
  it('GETs /api/requests/{id} and returns detail + answers', async () => {
    mockFetch.mockResolvedValue(
      okJson({ err: 0, request: { id: 3, user_id: 1, username: 'u', title: 't', body: 'b', seed_work_id: null, seed_fic: null, status: 'open', created_at: 'c', accepted_answer_id: null }, answers: [] }),
    );
    const { getRequest } = await importRequests();
    const res = await getRequest(3);
    expect(mockFetch.mock.calls[0][0]).toBe('/api/requests/3');
    expect(res.request.id).toBe(3);
  });
});

describe('createRequest', () => {
  it('POSTs title/body/seed_work_id with credentials', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, id: 9 }));
    const { createRequest } = await importRequests();
    const res = await createRequest({ title: 'New prompt', body: 'details', seed_work_id: 42 });
    expect(res.id).toBe(9);
    const [url, init] = mockFetch.mock.calls[0];
    expect(url).toBe('/api/requests');
    expect(init.method).toBe('POST');
    expect(init.credentials).toBe('include');
    const body = JSON.parse(init.body);
    expect(body.title).toBe('New prompt');
    expect(body.seed_work_id).toBe(42);
  });
});

describe('addAnswer', () => {
  it('POSTs work_id + url + pitch to /api/requests/{id}/answers', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, answer_id: 5 }));
    const { addAnswer } = await importRequests();
    const res = await addAnswer(3, 77, '', 'This fits!', null, null);
    expect(res.answer_id).toBe(5);
    const [url, init] = mockFetch.mock.calls[0];
    expect(url).toBe('/api/requests/3/answers');
    const body = JSON.parse(init.body);
    expect(body.work_id).toBe(77);
    expect(body.url).toBe('');
    expect(body.pitch).toBe('This fits!');
  });

  it('POSTs a URL answer with no work_id (URL-ingest)', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, answer_id: 6 }));
    const { addAnswer } = await importRequests();
    await addAnswer(3, null, 'https://archiveofourown.org/works/123', '', null, null);
    const [, init] = mockFetch.mock.calls[0];
    const body = JSON.parse(init.body);
    expect(body.work_id).toBeNull();
    expect(body.url).toBe('https://archiveofourown.org/works/123');
  });
});

describe('voteAnswer', () => {
  it('POSTs the vote to /api/requests/{id}/answers/{aid}/vote', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, score: 4, my_vote: 1 }));
    const { voteAnswer } = await importRequests();
    const res = await voteAnswer(3, 5, 1);
    expect(res.score).toBe(4);
    const [url, init] = mockFetch.mock.calls[0];
    expect(url).toBe('/api/requests/3/answers/5/vote');
    expect(JSON.parse(init.body).vote).toBe(1);
  });
});

describe('acceptAnswer', () => {
  it('POSTs an empty body to the accept endpoint', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0 }));
    const { acceptAnswer } = await importRequests();
    await acceptAnswer(3, 5);
    const [url, init] = mockFetch.mock.calls[0];
    expect(url).toBe('/api/requests/3/accept/5');
    expect(init.method).toBe('POST');
  });
});

describe('deleteRequest / deleteAnswer', () => {
  it('DELETEs the request', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0 }));
    const { deleteRequest } = await importRequests();
    await deleteRequest(3);
    const [url, init] = mockFetch.mock.calls[0];
    expect(url).toBe('/api/requests/3');
    expect(init.method).toBe('DELETE');
  });

  it('DELETEs the answer', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0 }));
    const { deleteAnswer } = await importRequests();
    await deleteAnswer(3, 5);
    const [url, init] = mockFetch.mock.calls[0];
    expect(url).toBe('/api/requests/3/answers/5');
    expect(init.method).toBe('DELETE');
  });
});
