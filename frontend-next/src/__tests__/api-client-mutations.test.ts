import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest';
import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';
import { handlers, mockSetlist, mockCrates } from '@/__mocks__/handlers';
import {
  generateSetlist,
  deleteSetlist,
  updateSetlist,
  duplicateSetlist,
  createCrate,
  deleteCrate,
  addSetlistToCrate,
  removeCrateTrack,
  refineSetlist,
  revertSetlist,
  importSpotifyPlaylist,
  checkSpotifyConnection,
  getSpotifyAuthUrl,
  arrangeSetlist,
  getImportStatus,
  getSetlistHistory,
  getCrate,
  ApiClientError,
} from '@/lib/api-client';

const server = setupServer(...handlers);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

describe('generateSetlist', () => {
  it('sends prompt and returns setlist', async () => {
    const result = await generateSetlist({ prompt: 'Deep house vibes' });
    expect(result.id).toBe('set-1');
    expect(result.tracks).toHaveLength(2);
  });

  it('sends optional parameters', async () => {
    const result = await generateSetlist({
      prompt: 'Techno',
      trackCount: 15,
      energyProfile: 'peak-time',
      creativeMode: true,
      bpmMin: 130,
      bpmMax: 145,
      verify: true,
    });
    expect(result.id).toBe('set-1');
  });

  it('works with minimal options', async () => {
    const result = await generateSetlist({ prompt: 'Chill' });
    expect(result.prompt).toBe('Deep house vibes');
  });
});

describe('deleteSetlist', () => {
  it('resolves on 204', async () => {
    await expect(deleteSetlist('set-1')).resolves.toBeUndefined();
  });
});

describe('updateSetlist', () => {
  it('returns updated setlist', async () => {
    const result = await updateSetlist('set-1', 'Renamed');
    expect(result.id).toBe('set-1');
  });
});

describe('duplicateSetlist', () => {
  it('returns duplicated setlist with new id', async () => {
    const result = await duplicateSetlist('set-1');
    expect(result.id).toBe('set-dup');
  });
});

describe('arrangeSetlist', () => {
  it('returns arranged setlist', async () => {
    const result = await arrangeSetlist('set-1', 'journey');
    expect(result.id).toBe('set-1');
  });
});

describe('createCrate', () => {
  it('returns created crate', async () => {
    const result = await createCrate('My Crate');
    expect(result.name).toBe('Weekend Set');
    expect(result.id).toBe('crate-1');
  });
});

describe('deleteCrate', () => {
  it('resolves on 204', async () => {
    await expect(deleteCrate('crate-1')).resolves.toBeUndefined();
  });
});

describe('getCrate', () => {
  it('returns crate detail with tracks', async () => {
    const result = await getCrate('crate-1');
    expect(result.name).toBe('Weekend Set');
    expect(result.tracks).toHaveLength(1);
  });
});

describe('addSetlistToCrate', () => {
  it('resolves on 204', async () => {
    await expect(addSetlistToCrate('crate-1', 'set-1')).resolves.toBeUndefined();
  });
});

describe('removeCrateTrack', () => {
  it('resolves on 204', async () => {
    await expect(removeCrateTrack('crate-1', 'ct-1')).resolves.toBeUndefined();
  });
});

describe('refineSetlist', () => {
  it('returns refinement response', async () => {
    const result = await refineSetlist('set-1', 'More energy please');
    expect(result.version_number).toBe(2);
    expect(result.explanation).toBe('Added more energy');
    expect(result.tracks).toHaveLength(2);
  });
});

describe('revertSetlist', () => {
  it('returns reverted version', async () => {
    const result = await revertSetlist('set-1', 1);
    expect(result.version_number).toBe(1);
    expect(result.explanation).toBe('Reverted to v1');
  });
});

describe('getSetlistHistory', () => {
  it('returns versions and conversation', async () => {
    const result = await getSetlistHistory('set-1');
    expect(result.versions).toHaveLength(1);
    expect(result.conversation).toHaveLength(2);
    expect(result.conversation[0].role).toBe('user');
  });
});

describe('importSpotifyPlaylist', () => {
  it('returns import id and status', async () => {
    const result = await importSpotifyPlaylist('https://open.spotify.com/playlist/abc');
    expect(result.import_id).toBe('imp-1');
    expect(result.status).toBe('complete');
  });
});

describe('getImportStatus', () => {
  it('returns import status details', async () => {
    const result = await getImportStatus('imp-1');
    expect(result.status).toBe('complete');
    expect(result.total).toBe(50);
  });
});

describe('checkSpotifyConnection', () => {
  it('returns connected status', async () => {
    const result = await checkSpotifyConnection('user-1');
    expect(result).toBe(false);
  });
});

describe('getSpotifyAuthUrl', () => {
  it('returns redirect url', async () => {
    const result = await getSpotifyAuthUrl('user-1');
    expect(result).toContain('accounts.spotify.com');
  });
});

describe('error handling', () => {
  it('throws ApiClientError with code on 400', async () => {
    server.use(
      http.post('/api/setlists/generate', () =>
        HttpResponse.json(
          { error: { code: 'INVALID_PROMPT', message: 'Prompt is empty' } },
          { status: 400 },
        ),
      ),
    );
    await expect(generateSetlist({ prompt: '' })).rejects.toThrow(ApiClientError);
    try {
      await generateSetlist({ prompt: '' });
    } catch (e) {
      expect(e).toBeInstanceOf(ApiClientError);
      expect((e as ApiClientError).code).toBe('INVALID_PROMPT');
      expect((e as ApiClientError).message).toBe('Prompt is empty');
    }
  });

  it('throws ApiClientError with UNKNOWN code on non-JSON error', async () => {
    server.use(
      http.post('/api/setlists/generate', () =>
        new HttpResponse('Internal Server Error', { status: 500 }),
      ),
    );
    try {
      await generateSetlist({ prompt: 'test' });
    } catch (e) {
      expect(e).toBeInstanceOf(ApiClientError);
      expect((e as ApiClientError).code).toBe('UNKNOWN');
    }
  });

  it('throws ApiClientError on 404', async () => {
    server.use(
      http.get('/api/setlists/:id', () =>
        HttpResponse.json(
          { error: { code: 'NOT_FOUND', message: 'Setlist not found' } },
          { status: 404 },
        ),
      ),
    );
    const { getSetlist } = await import('@/lib/api-client');
    await expect(getSetlist('nonexistent')).rejects.toThrow(ApiClientError);
  });

  it('throws ApiClientError on 422', async () => {
    server.use(
      http.post('/api/crates', () =>
        HttpResponse.json(
          { error: { code: 'VALIDATION_ERROR', message: 'Name too short' } },
          { status: 422 },
        ),
      ),
    );
    try {
      await createCrate('');
    } catch (e) {
      expect(e).toBeInstanceOf(ApiClientError);
      expect((e as ApiClientError).code).toBe('VALIDATION_ERROR');
    }
  });

  it('throws ApiClientError on 409 conflict', async () => {
    server.use(
      http.post('/api/crates/:crateId/setlists/:setlistId', () =>
        HttpResponse.json(
          { error: { code: 'DUPLICATE', message: 'Already in crate' } },
          { status: 409 },
        ),
      ),
    );
    await expect(addSetlistToCrate('crate-1', 'set-1')).rejects.toThrow(ApiClientError);
  });
});
