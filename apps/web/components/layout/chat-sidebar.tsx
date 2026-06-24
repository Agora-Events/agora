"use client";

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
  return (
    <div className="w-full max-w-sm bg-white rounded-2xl border border-border-warm shadow-[-4px_4px_0_rgba(0,0,0,1)] overflow-hidden">
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

      {/* Body */}
      {conversations.length > 0 ? (
        <ul className="divide-y divide-border-warm">
          {conversations.map((conv) => (
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
          title="No conversations yet"
          description="Start a conversation with an organizer or attendee to connect."
          action={onNewChat ? { label: "Start a Chat", onClick: onNewChat } : undefined}
        />
      )}
    </div>
  );
}
