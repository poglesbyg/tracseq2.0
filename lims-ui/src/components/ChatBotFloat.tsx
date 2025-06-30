import React from 'react';
import { ChatBubbleLeftIcon, QuestionMarkCircleIcon } from '@heroicons/react/24/outline';

interface ChatBotFloatProps {
  onClick: () => void;
  hasUnread?: boolean;
}

export const ChatBotFloat: React.FC<ChatBotFloatProps> = ({ 
  onClick, 
  hasUnread = false 
}) => {
  return (
    <button
      onClick={onClick}
      className="fixed bottom-4 right-4 w-14 h-14 bg-blue-600 text-white rounded-full shadow-lg hover:bg-blue-700 transition-all duration-300 hover:scale-110 focus:outline-none focus:ring-4 focus:ring-blue-200 z-40 group"
      aria-label="Open lab assistant chat"
    >
      <div className="relative flex items-center justify-center w-full h-full">
        <ChatBubbleLeftIcon className="w-6 h-6" />
        
        {/* Unread indicator */}
        {hasUnread && (
          <div className="absolute -top-1 -right-1 w-4 h-4 bg-red-500 rounded-full flex items-center justify-center">
            <div className="w-2 h-2 bg-white rounded-full"></div>
          </div>
        )}
        
        {/* Pulse animation */}
        <div className="absolute inset-0 rounded-full bg-blue-600 animate-ping opacity-20"></div>
        
        {/* Tooltip */}
        <div className="absolute bottom-full right-0 mb-2 px-3 py-1 bg-gray-800 text-white text-xs rounded-md opacity-0 group-hover:opacity-100 transition-opacity duration-200 whitespace-nowrap">
          <div className="flex items-center space-x-1">
            <QuestionMarkCircleIcon className="w-3 h-3" />
            <span>Ask Lab Assistant</span>
          </div>
          <div className="absolute top-full right-2 w-0 h-0 border-l-2 border-r-2 border-t-4 border-transparent border-t-gray-800"></div>
        </div>
      </div>
    </button>
  );
}; 
