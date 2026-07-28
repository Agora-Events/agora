"use client";

import { useState, useMemo } from "react";
import Image from "next/image";
import { EmptyState } from "@/components/ui/empty-state";

interface Conversation {
  id: string;
  name: string;
  lastMessage: string;
  time: string;
  avatarUrl?: string;
}

interface ChatSidebarProps {
  conversations?: Conversation[];
  onNewChat?: () => void;
}

export function ChatSidebar({ conversations = [], onNewChat }: ChatSidebarProps) {
  const [searchQuery, setSearchQuery] = useState("");

  const filteredConversations = useMemo(() => {
    if (!searchQuery.trim()) return conversations;
    const q = searchQuery.toLowerCase();
    return conversations.filter(
      (conv) =>
        conv.name.toLowerCase().includes(q) ||
        conv.lastMessage.toLowerCase().includes(q)
    );
  }, [conversations, searchQuery]);

  return (
    <div className="w-full md:max-w-sm bg-white rounded-2xl border border-border-warm shadow-[-4px_4px_0_rgba(0,0,0,1)] overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between px-5 py-4 border-b border-border-warm">
        <h3 className="font-semibold text-ink-deep text-base">Messages</h3>
        {onNewChat && (
          <button
            type="button"
            onClick={onNewChat}
            className="w-8 h-8 flex items-center justify-center rounded-full bg-surface hover:bg-surface-alt transition-colors"
            aria-label="New chat"
          >
            <Image src="/icons/add-circle.svg" width={18} height={18} alt="new chat" />
          </button>
        )}
      </div>

      {/* Search / Filter */}
      {conversations.length > 0 && (
        <div className="px-4 py-3 border-b border-border-warm">
          <input
            type="search"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search conversations…"
            aria-label="Filter conversations"
            className="w-full rounded-lg border border-border-warm bg-surface px-3 py-2 text-sm text-ink-deep placeholder:text-ink-deep/40 focus:outline-none focus:ring-2 focus:ring-primary/50"
          />
        </div>
      )}

      {/* Body */}
      {filteredConversations.length > 0 ? (
        <ul className="divide-y divide-border-warm">
          {filteredConversations.map((conv) => (
            <li
              key={conv.id}
              className="flex items-center gap-3 px-5 py-3 hover:bg-surface/50 cursor-pointer transition-colors"
            >
              <div className="w-10 h-10 rounded-full bg-surface flex-shrink-0 overflow-hidden">
                {conv.avatarUrl && (
                  <Image src={conv.avatarUrl} alt={conv.name} width={40} height={40} className="object-cover" />
                )}
              </div>
              <div className="flex-1 min-w-0">
                <p className="text-sm font-medium text-ink-deep truncate">{conv.name}</p>
                <p className="text-xs text-ink-deep/50 truncate">{conv.lastMessage}</p>
              </div>
              <span className="text-xs text-ink-deep/40 flex-shrink-0">{conv.time}</span>
            </li>
          ))}
        </ul>
      ) : (
        <EmptyState
          icon={
            <Image
              src="/icons/bubble-chat.svg"
              width={32}
              height={32}
              alt="no messages"
            />
          }
          title={searchQuery ? "No conversations found" : "No conversations yet"}
          description={
            searchQuery
              ? "No conversations match your search. Try a different name or message."
              : "Start a conversation with an organizer or attendee to connect."
          }
          action={!searchQuery && onNewChat ? { label: "Start a Chat", onClick: onNewChat } : undefined}
        />
      )}
    </div>
  );
}
