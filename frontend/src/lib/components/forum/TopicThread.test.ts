
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
globalThis.fetch = vi.fn() as unknown as typeof fetch;
const mockFetch = globalThis.fetch as unknown as ReturnType<typeof vi.fn>;
import { auth } from '$lib/stores/auth.svelte';
function jsonResponse(body: unknown){ return { ok:true, json: async()=>body };}
beforeEach(()=>{ mockFetch.mockReset(); localStorage.clear(); auth.user={id:1,username:'tester',level:0} as any; });
function topicDetail(status='open'){
 return { err:0, id:5, title:'Topic', topic_slug:'topic-5', author_id:2, author_username:'mod', category_slug:'general', category_title:'General', status, body:'hello', payload:{}, view_count:10, created_at:'2026-08-10T00:00:00Z', updated_at:'2026-08-10T00:00:00Z', items:[{id:10, author_id:2, author_username:'mod', body:'hello', quote_of:null, quote:null, edited_at:null, deleted_at:null, created_at:'2026-08-10T00:00:00Z', score:0, is_op:true}], next_cursor:null, limit:25, view_count_before:9 };
}
describe('TopicThread pin/lock',()=>{
 it('shows locked badge and hides reply for non-curator when locked',async()=>{
   auth.user={id:1,username:'tester',level:0} as any;
   mockFetch.mockImplementation((u:RequestInfo|URL)=>{ const s=String(u); if(s.includes('/api/forum/topics/5')) return Promise.resolve(jsonResponse(topicDetail('locked'))); if(s.includes('/api/forum/topics/5/follow')) return Promise.resolve(jsonResponse({err:0,following:false,follower_count:0})); return Promise.resolve(jsonResponse({err:0,items:[]})); });
   const { default: Comp } = await import('./TopicThread.svelte');
   render(Comp,{props:{topicId:5}});
   await waitFor(()=> expect(screen.getByText('Topic')).toBeTruthy());
   expect(screen.getByText('Locked')).toBeTruthy();
   expect(screen.queryByPlaceholderText(/reply/i)).toBeNull();
 });
 it('shows pin/lock toggles for curator',async()=>{
   auth.user={id:1,username:'tester',role:10,level:50} as any;
   mockFetch.mockImplementation((u:RequestInfo|URL)=>{ const s=String(u); if(s.includes('/api/forum/topics/5')) return Promise.resolve(jsonResponse(topicDetail('open'))); if(s.includes('/api/forum/topics/5/follow')) return Promise.resolve(jsonResponse({err:0,following:false,follower_count:0})); return Promise.resolve(jsonResponse({err:0,items:[]})); });
   const { default: Comp } = await import('./TopicThread.svelte');
   render(Comp,{props:{topicId:5}});
   await waitFor(()=> expect(screen.getByText('Topic')).toBeTruthy());
   expect(screen.getByText('Pin')).toBeTruthy();
   expect(screen.getByText('Lock')).toBeTruthy();
 });
});
