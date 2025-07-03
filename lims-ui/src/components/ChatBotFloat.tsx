import React, { useState, useEffect } from 'react';
import { ChatBubbleLeftIcon, SparklesIcon } from '@heroicons/react/24/outline';

interface ChatBotFloatProps {
  onClick: () => void;
  hasUnread?: boolean;
}

export const ChatBotFloat: React.FC<ChatBotFloatProps> = ({ 
  onClick, 
  hasUnread = false 
}) => {
  const [showTooltip, setShowTooltip] = useState(false);
  const [isFirstVisit, setIsFirstVisit] = useState(false);

  useEffect(() => {
    // Check if it's the user's first visit
    const hasVisited = localStorage.getItem('chatbot_visited');
    if (!hasVisited) {
      setIsFirstVisit(true);
      // Show tooltip after a delay on first visit
      const timer = setTimeout(() => {
        setShowTooltip(true);
        // Hide after 5 seconds
        setTimeout(() => {
          setShowTooltip(false);
          localStorage.setItem('chatbot_visited', 'true');
        }, 5000);
      }, 2000);
      return () => clearTimeout(timer);
    }
  }, []);

  return (
    <div className="fixed bottom-4 right-4 z-40">
      {/* Tooltip */}
      {(showTooltip || isFirstVisit) && (
        <div className="absolute bottom-full right-0 mb-2 animate-fadeIn">
          <div className="bg-gray-800 text-white text-sm rounded-lg px-4 py-2 shadow-lg max-w-xs">
            <div className="flex items-center space-x-2">
              <SparklesIcon className="w-4 h-4 text-yellow-300" />
              <span className="font-medium">Need help?</span>
            </div>
            <p className="mt-1 text-xs text-gray-300">
              I'm your AI lab assistant. Ask me anything about samples, protocols, or lab procedures!
            </p>
            <div className="absolute top-full right-4 w-0 h-0 border-l-4 border-r-4 border-t-4 border-transparent border-t-gray-800"></div>
          </div>
        </div>
      )}

      {/* Float Button */}
      <button
        onClick={onClick}
        onMouseEnter={() => !isFirstVisit && setShowTooltip(true)}
        onMouseLeave={() => !isFirstVisit && setShowTooltip(false)}
        className="relative group"
        aria-label="Open AI lab assistant chat"
      >
        {/* Main Button */}
        <div className="relative w-14 h-14 bg-gradient-to-br from-blue-600 to-purple-600 text-white rounded-full shadow-lg hover:shadow-xl transform transition-all duration-300 hover:scale-110 focus:outline-none focus:ring-4 focus:ring-purple-200 overflow-hidden">
          {/* Animated gradient background */}
          <div className="absolute inset-0 bg-gradient-to-br from-purple-600 to-blue-600 opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
          
          {/* Icon container */}
          <div className="relative flex items-center justify-center w-full h-full">
            <ChatBubbleLeftIcon className="w-6 h-6 transition-transform group-hover:scale-110" />
            <SparklesIcon className="w-3 h-3 absolute -top-0.5 -right-0.5 text-yellow-300" />
          </div>
          
          {/* Unread indicator */}
          {hasUnread && (
            <div className="absolute -top-1 -right-1 flex items-center justify-center">
              <span className="relative flex h-3 w-3">
                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-red-400 opacity-75"></span>
                <span className="relative inline-flex rounded-full h-3 w-3 bg-red-500"></span>
              </span>
            </div>
          )}
        </div>
        
        {/* Pulse animation for attention */}
        <div className="absolute inset-0 rounded-full bg-gradient-to-br from-blue-600 to-purple-600 animate-pulse opacity-30 scale-105"></div>
        
        {/* Ripple effect on hover */}
        <div className="absolute inset-0 rounded-full bg-white opacity-0 group-hover:animate-ripple"></div>
      </button>
    </div>
  );
}; 
