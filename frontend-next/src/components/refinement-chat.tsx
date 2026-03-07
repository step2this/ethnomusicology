'use client';

import { useState, useRef, useEffect } from 'react';
import { Send, Loader2, AlertTriangle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useSetlistHistory, useRefineSetlist } from '@/hooks/use-refinement';

export function RefinementChat({ setlistId }: { setlistId: string }) {
  const [message, setMessage] = useState('');
  const scrollRef = useRef<HTMLDivElement>(null);
  const { data: history } = useSetlistHistory(setlistId);
  const refineMutation = useRefineSetlist();

  const conversation = history?.conversation ?? [];

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [conversation.length, refineMutation.isPending]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const trimmed = message.trim();
    if (!trimmed || refineMutation.isPending) return;
    setMessage('');
    refineMutation.mutate({ setlistId, message: trimmed });
  };

  return (
    <div className="flex h-full flex-col rounded-lg border border-border bg-card">
      <div className="border-b border-border px-4 py-2">
        <h3 className="text-sm font-medium text-foreground">Refine Setlist</h3>
      </div>

      {/* Messages */}
      <div ref={scrollRef} className="flex-1 overflow-y-auto p-3 space-y-2">
        {conversation.length === 0 && !refineMutation.isPending && (
          <p className="text-center text-sm text-muted-foreground py-8">
            Type a message to refine your setlist
          </p>
        )}

        {conversation.map((msg, i) => (
          <div
            key={i}
            className={`flex ${msg.role === 'user' ? 'justify-end' : 'justify-start'}`}
          >
            <div
              className={`max-w-[85%] rounded-xl px-3 py-2 text-sm ${
                msg.role === 'user'
                  ? 'bg-primary/20 text-foreground'
                  : 'bg-muted text-foreground'
              }`}
            >
              {msg.content}
            </div>
          </div>
        ))}

        {refineMutation.isPending && (
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <Loader2 className="h-4 w-4 animate-spin" />
            <span>Refining...</span>
          </div>
        )}

        {refineMutation.data?.change_warning && (
          <div className="flex items-center gap-2 rounded-lg bg-amber-900/30 border border-amber-700/50 px-3 py-2 text-xs text-amber-300">
            <AlertTriangle className="h-3.5 w-3.5 shrink-0" />
            <span>{refineMutation.data.change_warning}</span>
          </div>
        )}
      </div>

      {/* Input */}
      <form onSubmit={handleSubmit} className="flex items-center gap-2 border-t border-border p-3">
        <input
          type="text"
          value={message}
          onChange={(e) => setMessage(e.target.value)}
          placeholder="e.g. swap track 3 for something deeper..."
          className="flex-1 rounded-md bg-muted px-3 py-1.5 text-sm text-foreground placeholder:text-muted-foreground outline-none focus:ring-1 focus:ring-primary"
          disabled={refineMutation.isPending}
        />
        <Button
          type="submit"
          size="icon-sm"
          disabled={!message.trim() || refineMutation.isPending}
        >
          <Send className="h-3.5 w-3.5" />
        </Button>
      </form>
    </div>
  );
}
