import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest';
import { setupServer } from 'msw/node';
import { handlers } from '@/__mocks__/handlers';
import {
  listSetlists,
  getSetlist,
  listCrates,
  getPurchaseLinks,
  listTracks,
  searchPreview,
} from '@/lib/api-client';

const server = setupServer(...handlers);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

describe('api-client', () => {
  it('listSetlists returns summaries', async () => {
    const result = await listSetlists();
    expect(result).toHaveLength(2);
    expect(result[0].name).toBe('Test Setlist');
  });

  it('getSetlist returns full setlist', async () => {
    const result = await getSetlist('set-1');
    expect(result.id).toBe('set-1');
    expect(result.tracks).toHaveLength(2);
    expect(result.tracks[0].title).toBe('Strings of Life');
  });

  it('listCrates returns crates', async () => {
    const result = await listCrates();
    expect(result).toHaveLength(1);
    expect(result[0].name).toBe('Weekend Set');
  });

  it('getPurchaseLinks returns store links', async () => {
    const result = await getPurchaseLinks('Strings of Life', 'Derrick May');
    expect(result).toHaveLength(4);
    expect(result[0].store).toBe('beatport');
  });

  it('listTracks returns paginated response', async () => {
    const result = await listTracks();
    expect(result.tracks).toHaveLength(1);
    expect(result.page).toBe(1);
    expect(result.total).toBe(1);
  });

  it('searchPreview returns preview data', async () => {
    const result = await searchPreview('Strings of Life', 'Derrick May');
    expect(result.source).toBe('deezer');
    expect(result.preview_url).toBeTruthy();
  });

  it('searchPreview returns fallback on error', async () => {
    // Override with error handler
    const { http, HttpResponse } = await import('msw');
    server.use(
      http.get('/api/audio/search', () =>
        HttpResponse.json({ error: { code: 'NOT_FOUND', message: 'nope' } }, { status: 404 }),
      ),
    );
    const result = await searchPreview('Unknown', 'Artist');
    expect(result.source).toBeNull();
    expect(result.preview_url).toBeNull();
  });
});
