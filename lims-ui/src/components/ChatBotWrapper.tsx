import React from 'react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ChatBot } from './ChatBot'

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5, // 5 minutes
      gcTime: 1000 * 60 * 10, // 10 minutes
    },
  },
})

interface ChatBotWrapperProps {
  isOpen: boolean
  onToggle: () => void
}

export const ChatBotWrapper: React.FC<ChatBotWrapperProps> = ({ isOpen, onToggle }) => {
  return (
    <QueryClientProvider client={queryClient}>
      <ChatBot isOpen={isOpen} onToggle={onToggle} />
    </QueryClientProvider>
  )
} 