import React, { useState, useRef, useEffect, useCallback } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { useSpeechRecognition } from 'react-speech-kit';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { 
  PaperAirplaneIcon, 
  ChatBubbleLeftRightIcon, 
  UserIcon, 
  MinusIcon, 
  PlusIcon, 
  XMarkIcon,
  SparklesIcon,
  BeakerIcon,
  DocumentTextIcon,
  ClipboardDocumentListIcon,
  ArrowPathIcon,
  MicrophoneIcon,
  DocumentMagnifyingGlassIcon,
  ChartBarIcon
} from '@heroicons/react/24/outline';

interface Message {
  id: string;
  content: string;
  type: 'user' | 'assistant' | 'system';
  timestamp: Date;
  isTyping?: boolean;
  confidence?: number;
  metadata?: {
    confidence?: number;
    sources?: string[];
    sampleIds?: string[];
    modelUsed?: string;
    processingTime?: number;
    sourceContext?: string[];
    action?: {
      type: string;
      payload: any;
    };
  };
  attachments?: Array<{
    name: string;
    type: string;
    url: string;
  }>;
  actions?: Array<{
    label: string;
    action: string;
    variant: 'primary' | 'secondary' | 'danger';
    data?: any;
  }>;
  isStreaming?: boolean;
}

interface ChatBotProps {
  isOpen: boolean;
  onToggle: () => void;
}

interface QuickAction {
  label: string;
  icon: React.ComponentType<any>;
  prompt: string;
  description: string;
}

const quickActions: QuickAction[] = [
  {
    label: 'Create Sample',
    icon: BeakerIcon,
    prompt: 'I need to create a new sample. Can you guide me through the process?',
    description: 'Register new laboratory sample'
  },
  {
    label: 'Process PDF',
    icon: DocumentTextIcon,
    prompt: 'I have a laboratory submission PDF to process and extract information from.',
    description: 'Extract data from submission forms'
  },
  {
    label: 'View Protocols',
    icon: DocumentMagnifyingGlassIcon,
    prompt: 'Show me the available laboratory protocols and standard operating procedures.',
    description: 'Browse SOPs and protocols'
  },
  {
    label: 'Generate Report',
    icon: ChartBarIcon,
    prompt: 'I need to generate a laboratory report. What options are available?',
    description: 'Create various lab reports'
  },
];

export const ChatBot: React.FC<ChatBotProps> = ({ isOpen, onToggle }) => {
  const [sessionId] = useState(() => `chat_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`);
  const [messages, setMessages] = useState<Message[]>([
    {
      id: '1',
      content: "👋 Hi! I'm your AI lab assistant powered by advanced RAG technology.\n\nI can help you with:\n\n🧪 **Sample Management**\n• Process submissions and track samples\n• Understand storage requirements\n• Generate barcodes and labels\n\n📋 **Laboratory Workflows**\n• Guide you through protocols\n• Answer questions about procedures\n• Help with quality control\n\n🤖 **AI-Powered Features**\n• Extract data from PDFs and documents\n• Search through lab knowledge base\n• Provide intelligent recommendations\n\n💡 **Tips**: You can drag files directly into this chat or use quick actions below!",
      type: 'assistant',
      timestamp: new Date(),
    },
  ]);
  const [inputValue, setInputValue] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [isMinimized, setIsMinimized] = useState(false);
  const [connectionStatus, setConnectionStatus] = useState<'connected' | 'connecting' | 'disconnected'>('connected');
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const conversationId = useRef<string>('');
  const uploadedFiles = useRef<File[]>([]);
  const queryClient = useQueryClient();
  
  // WebSocket state
  const [wsConnection, setWsConnection] = useState<WebSocket | null>(null);
  const [wsConnected, setWsConnected] = useState(false);

  // Voice input setup
  const { listen, listening, stop } = useSpeechRecognition({
    onResult: (result: string) => {
      setInputValue(result);
    },
  });

  // Load conversation history
  const { data: conversationHistory } = useQuery({
    queryKey: ['chatHistory', conversationId.current],
    queryFn: async () => {
      const storedHistory = localStorage.getItem(`chat_history_${conversationId.current}`);
      if (storedHistory) {
        return JSON.parse(storedHistory);
      }
      return [];
    },
    enabled: !!conversationId.current
  });

  // Save conversation to localStorage whenever messages change
  useEffect(() => {
    if (conversationId.current && messages.length > 0) {
      localStorage.setItem(`chat_history_${conversationId.current}`, JSON.stringify(messages));
    }
  }, [messages, conversationId]);

  // Initialize conversation ID
  useEffect(() => {
    const id = `conv_${Date.now()}_${Math.random().toString(36).substring(7)}`;
    conversationId.current = id;
  }, []);

  // Load previous messages if available
  useEffect(() => {
    if (conversationHistory && conversationHistory.length > 0) {
      setMessages(conversationHistory);
    } else {
      // Add welcome message
      setMessages([{
        id: '1',
        content: `# 🧪 Welcome to TracSeq AI Assistant!

I'm here to help you with:
- Sample registration and tracking
- Laboratory submission processing
- Protocol guidance and SOPs
- Report generation
- Data analysis and insights

**Quick tip:** You can upload PDFs for automatic data extraction, use voice input, or choose from the quick actions below.

How can I assist you today?`,
        type: 'assistant',
        timestamp: new Date(),
        confidence: 1.0
      }]);
    }
  }, [conversationHistory]);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  useEffect(() => {
    if (isOpen && !isMinimized) {
      inputRef.current?.focus();
    }
  }, [isOpen, isMinimized]);

  // Check connection status
  useEffect(() => {
    const checkConnection = async () => {
      try {
        const response = await fetch('/api/health');
        if (response.ok) {
          setConnectionStatus('connected');
        } else {
          setConnectionStatus('disconnected');
        }
      } catch {
        setConnectionStatus('disconnected');
      }
    };

    checkConnection();
    const interval = setInterval(checkConnection, 30000); // Check every 30 seconds
    return () => clearInterval(interval);
  }, []);

  // Initialize WebSocket connection
  useEffect(() => {
    if (isOpen && conversationId.current) {
      // Get auth token from localStorage
      const token = localStorage.getItem('authToken') || '';
      
      // Connect to WebSocket
      const ws = new WebSocket(`ws://localhost:8089/ws/chat/${conversationId.current}?token=${token}`);
      
      ws.onopen = () => {
        console.log('WebSocket connected');
        setWsConnected(true);
      };
      
      ws.onmessage = (event) => {
        const data = JSON.parse(event.data);
        
        switch (data.type) {
          case 'connection':
            console.log('WebSocket connection confirmed:', data);
            break;
            
          case 'message':
            // Only add messages from other users (not echo)
            if (data.status !== 'sent') {
              const wsMessage: Message = {
                id: Date.now().toString(),
                content: data.content,
                type: data.user.id === 'assistant' ? 'assistant' : 'user',
                timestamp: new Date(data.timestamp),
                metadata: {
                  modelUsed: 'websocket',
                  sourceContext: ['real-time']
                }
              };
              setMessages(prev => [...prev, wsMessage]);
            }
            break;
            
          case 'typing':
            // Handle typing indicators if needed
            console.log('User typing:', data.user.name, data.typing);
            break;
            
          case 'user_disconnected':
            console.log('User disconnected:', data.user);
            break;
        }
      };
      
      ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        setWsConnected(false);
      };
      
      ws.onclose = () => {
        console.log('WebSocket disconnected');
        setWsConnected(false);
      };
      
      setWsConnection(ws);
      
      return () => {
        if (ws.readyState === WebSocket.OPEN) {
          ws.close();
        }
      };
    }
  }, [isOpen, conversationId.current]);

  // File upload handler
  const handleFileUpload = useCallback((files: FileList) => {
    const fileArray = Array.from(files);
    uploadedFiles.current = [...uploadedFiles.current, ...fileArray];
    
    // Create attachment messages
    fileArray.forEach(file => {
      const fileMessage: Message = {
        id: Date.now().toString(),
        content: `📎 Uploaded: ${file.name} (${(file.size / 1024).toFixed(2)} KB)`,
        type: 'user',
        timestamp: new Date(),
        attachments: [{
          name: file.name,
          type: file.type,
          url: URL.createObjectURL(file)
        }]
      };
      setMessages(prev => [...prev, fileMessage]);
    });
  }, []);

  // Send message mutation with streaming support
  const sendMessageMutation = useMutation({
    mutationFn: async ({ message, attachments }: { message: string, attachments?: File[] }) => {
      const formData = new FormData();
      formData.append('message', message);
      formData.append('conversationId', conversationId.current);
      
      if (attachments) {
        attachments.forEach(file => {
          formData.append('files', file);
        });
      }

      // Use EventSource for Server-Sent Events streaming
      // This is a placeholder - in production, you'd use EventSource or fetch with streams
      const response = await fetch('/api/chat/stream', {
        method: 'POST',
        body: formData,
      });

      if (!response.ok) {
        throw new Error('Failed to get chat response');
      }

      return response;
    },
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['chatHistory'] });
    }
  });

  // Simulate streaming response
  const simulateStreamingResponse = (content: string, messageId: string) => {
    const words = content.split(' ');
    let currentContent = '';
    let wordIndex = 0;

    const streamInterval = setInterval(() => {
      if (wordIndex < words.length) {
        currentContent += (wordIndex > 0 ? ' ' : '') + words[wordIndex];
        setMessages(prev => prev.map(msg => 
          msg.id === messageId 
            ? { ...msg, content: currentContent, isStreaming: true }
            : msg
        ));
        wordIndex++;
      } else {
        clearInterval(streamInterval);
        setMessages(prev => prev.map(msg => 
          msg.id === messageId 
            ? { ...msg, isStreaming: false }
            : msg
        ));
        setIsLoading(false);
      }
    }, 50);
  };

  const handleSend = async () => {
    if (!inputValue.trim() && uploadedFiles.current.length === 0) return;

    // Add user message
    const userMessage: Message = {
      id: Date.now().toString(),
      content: inputValue,
      type: 'user',
      timestamp: new Date(),
      attachments: uploadedFiles.current.map(file => ({
        name: file.name,
        type: file.type,
        url: URL.createObjectURL(file)
      }))
    };

    setMessages(prev => [...prev, userMessage]);
    const currentInput = inputValue;
    const currentFiles = [...uploadedFiles.current];
    setInputValue('');
    setIsLoading(true);
    
    // Send via WebSocket for real-time collaboration
    if (wsConnected && wsConnection) {
      wsConnection.send(JSON.stringify({
        type: 'message',
        content: currentInput
      }));
    }

    try {
      // Create FormData for the request
      const formData = new FormData();
      formData.append('message', currentInput);
      formData.append('conversationId', conversationId.current);
      
      // Append files if any
      currentFiles.forEach(file => {
        formData.append('files', file);
      });

      // Create bot message placeholder
      const botMessageId = (Date.now() + 1).toString();
      setMessages(prev => [...prev, {
        id: botMessageId,
        content: '',
        type: 'assistant',
        timestamp: new Date(),
        isStreaming: true
      }]);

      // Use EventSource for Server-Sent Events
      const response = await fetch('/api/chat/stream', {
        method: 'POST',
        body: formData,
      });

      if (!response.ok) {
        throw new Error('Failed to get chat response');
      }

      // Read the streaming response
      const reader = response.body?.getReader();
      const decoder = new TextDecoder();
      let buffer = '';

      if (reader) {
        while (true) {
          const { done, value } = await reader.read();
          if (done) break;

          buffer += decoder.decode(value, { stream: true });
          const lines = buffer.split('\n\n');
          buffer = lines.pop() || '';

          for (const line of lines) {
            if (line.startsWith('data: ')) {
              const data = line.slice(6);
              if (data === '[DONE]') {
                setMessages(prev => prev.map(msg => 
                  msg.id === botMessageId 
                    ? { ...msg, isStreaming: false }
                    : msg
                ));
                setIsLoading(false);
                break;
              }

              try {
                const parsed = JSON.parse(data);
                if (parsed.type === 'chunk') {
                  setMessages(prev => prev.map(msg => 
                    msg.id === botMessageId 
                      ? { ...msg, content: msg.content + parsed.content }
                      : msg
                  ));
                } else if (parsed.type === 'completion') {
                  setMessages(prev => prev.map(msg => 
                    msg.id === botMessageId 
                      ? { 
                          ...msg, 
                          isStreaming: false,
                          confidence: parsed.metadata?.confidence,
                          metadata: parsed.metadata
                        }
                      : msg
                  ));
                }
              } catch (e) {
                console.error('Error parsing SSE data:', e);
              }
            }
          }
        }
      }
    } catch (error) {
      console.error('Error sending message:', error);
      setIsLoading(false);
      
      // Show error message
      const errorMessage: Message = {
        id: (Date.now() + 2).toString(),
        content: '❌ Sorry, I encountered an error processing your request. Please try again.',
        type: 'system',
        timestamp: new Date()
      };
      setMessages(prev => [...prev, errorMessage]);
    }

    // Clear uploaded files after sending
    uploadedFiles.current = [];
  };

  const generateMockResponse = (userInput: string, files: File[]) => {
    const input = userInput.toLowerCase();
    
    if (files.length > 0 && files.some(f => f.type === 'application/pdf')) {
      return {
        content: `## 📄 PDF Processing Complete

I've successfully extracted information from **${files[0].name}**:

### Extracted Information:
- **Submitter:** Dr. Jane Smith
- **Institution:** Central Research Lab
- **Sample Type:** DNA Extract
- **Volume:** 50 µL
- **Concentration:** 125 ng/µL
- **Storage Requirements:** -20°C

### Quality Metrics:
- **A260/280:** 1.85 ✅
- **A260/230:** 2.10 ✅
- **RIN Score:** 9.2 ✅

Would you like me to:
1. Create a sample record with this information?
2. Generate storage labels?
3. Schedule quality control testing?`,
        confidence: 0.92,
        actions: [
          { label: 'Create Sample', action: 'create_sample', variant: 'primary' as const },
          { label: 'Generate Labels', action: 'generate_labels', variant: 'secondary' as const },
          { label: 'Schedule QC', action: 'schedule_qc', variant: 'secondary' as const }
        ],
        metadata: {
          modelUsed: 'gpt-4-vision',
          processingTime: 2.3,
          sourceContext: ['submission_form_v2.3', 'lab_protocols_2024']
        }
      };
    }

    if (input.includes('create') && input.includes('sample')) {
      return {
        content: `## 🧪 Sample Creation Wizard

Let me help you create a new sample. I'll need some information:

### Required Information:
1. **Sample Type**: 
   - [ ] DNA
   - [ ] RNA
   - [ ] Protein
   - [ ] Cell Culture
   - [ ] Other

2. **Sample Details**:
   - Volume: ___ µL
   - Concentration: ___ ng/µL
   - Buffer: ___

3. **Storage Location**:
   - Temperature: ___°C
   - Box/Rack: ___
   - Position: ___

4. **Project Association**:
   - Project ID: ___
   - Principal Investigator: ___

Would you like me to start with a specific sample type?`,
        confidence: 0.95,
        actions: [
          { label: 'DNA Sample', action: 'create_dna_sample', variant: 'primary' as const },
          { label: 'RNA Sample', action: 'create_rna_sample', variant: 'primary' as const },
          { label: 'Import from CSV', action: 'import_csv', variant: 'secondary' as const }
        ],
        metadata: {
          modelUsed: 'gpt-4',
          processingTime: 0.8
        }
      };
    }

    if (input.includes('protocol') || input.includes('sop')) {
      return {
        content: `## 📚 Laboratory Protocols

Here are the available protocols and SOPs:

### DNA/RNA Extraction
- **Protocol ID:** SOP-001
- **Version:** 2.3
- **Last Updated:** Jan 2024
- [View Protocol](# "Open protocol document")

### Library Preparation
- **Protocol ID:** SOP-005
- **Version:** 1.8
- **Last Updated:** Feb 2024
- [View Protocol](# "Open protocol document")

### Quality Control
- **Protocol ID:** SOP-009
- **Version:** 3.1
- **Last Updated:** Dec 2023
- [View Protocol](# "Open protocol document")

### Sample Storage
- **Protocol ID:** SOP-012
- **Version:** 2.0
- **Last Updated:** Jan 2024
- [View Protocol](# "Open protocol document")

Which protocol would you like to access?`,
        confidence: 1.0,
        actions: [
          { label: 'Download All', action: 'download_all_protocols', variant: 'primary' as const },
          { label: 'Search Protocols', action: 'search_protocols', variant: 'secondary' as const }
        ],
        metadata: {
          modelUsed: 'retrieval',
          sourceContext: ['protocol_database', 'sop_repository']
        }
      };
    }

    // Default response
    return {
      content: `I understand you're asking about "${userInput}". 

I can help you with:
- **Sample Management**: Registration, tracking, and storage
- **Document Processing**: Extract data from PDFs and forms
- **Protocol Access**: View and download laboratory SOPs
- **Report Generation**: Create various laboratory reports
- **Data Analysis**: Insights and quality metrics

Please provide more specific details about what you'd like to accomplish, or try one of the quick actions below.`,
      confidence: 0.85,
      actions: [
        { label: 'View Samples', action: 'view_samples', variant: 'secondary' as const },
        { label: 'Recent Reports', action: 'recent_reports', variant: 'secondary' as const }
      ],
      metadata: {
        modelUsed: 'gpt-3.5-turbo',
        processingTime: 0.5
      }
    };
  };

  const handleActionClick = async (action: any) => {
    console.log('Action clicked:', action);
    
    const actionMessage: Message = {
      id: Date.now().toString(),
      content: `⏳ Executing action: **${action.label}**...`,
      type: 'system',
      timestamp: new Date()
    };
    setMessages(prev => [...prev, actionMessage]);

    try {
      switch (action.action) {
        case 'create_sample':
          // Navigate to sample creation or open modal
          window.location.href = '/samples/new';
          break;
          
        case 'generate_labels':
          // Trigger label generation
          const labelResponse = await fetch('/api/samples/labels', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ sampleIds: [action.data?.sampleId] })
          });
          
          if (labelResponse.ok) {
            setMessages(prev => [...prev, {
              id: Date.now().toString(),
              content: '✅ Labels generated successfully! Check your downloads.',
              type: 'system',
              timestamp: new Date()
            }]);
          }
          break;
          
        case 'schedule_qc':
          // Navigate to QC scheduling
          window.location.href = `/qc/schedule?sampleId=${action.data?.sampleId}`;
          break;
          
        case 'download_all_protocols':
          // Download all protocols
          window.open('/api/protocols/download-all', '_blank');
          break;
          
        case 'view_samples':
          // Navigate to samples page
          window.location.href = '/samples';
          break;
          
        case 'recent_reports':
          // Navigate to reports page
          window.location.href = '/reports';
          break;
          
        default:
          console.warn('Unknown action:', action.action);
      }
    } catch (error) {
      console.error('Error executing action:', error);
      setMessages(prev => [...prev, {
        id: Date.now().toString(),
        content: `❌ Failed to execute action: ${error instanceof Error ? error.message : 'Unknown error'}`,
        type: 'system',
        timestamp: new Date()
      }]);
    }
  };

  const handleQuickAction = (action: QuickAction) => {
    setInputValue(action.prompt);
    // Automatically send the message
    setTimeout(() => {
      const button = document.querySelector('[data-send-button]') as HTMLButtonElement;
      button?.click();
    }, 100);
  };

  const toggleVoiceInput = () => {
    if (listening) {
      stop();
    } else {
      listen();
    }
  };

  if (!isOpen) return null;

  return (
    <div className={`fixed bottom-4 right-4 bg-white rounded-2xl shadow-2xl border border-gray-200 transition-all duration-300 ease-in-out ${
      isMinimized ? 'w-80 h-16' : 'w-[420px] h-[600px]'
    } z-50`}>
      {/* Header */}
      <div className="flex items-center justify-between p-4 bg-gradient-to-r from-blue-600 to-purple-600 text-white rounded-t-2xl">
        <div className="flex items-center space-x-3">
          <div className="relative">
            <ChatBubbleLeftRightIcon className="w-6 h-6" />
            <SparklesIcon className="w-3 h-3 absolute -bottom-1 -right-1 text-yellow-300" />
          </div>
          <div>
            <h3 className="font-semibold">Lab AI Assistant</h3>
            <div className="flex items-center text-xs opacity-90">
              <div className={`w-2 h-2 rounded-full mr-1 ${
                connectionStatus === 'connected' ? 'bg-green-400' : 
                connectionStatus === 'connecting' ? 'bg-yellow-400 animate-pulse' : 
                'bg-red-400'
              }`}></div>
              {connectionStatus === 'connected' ? 'Connected' : 
               connectionStatus === 'connecting' ? 'Connecting...' : 
               'Disconnected'}
            </div>
          </div>
        </div>
        <div className="flex items-center space-x-2">
          <button
            onClick={() => setIsMinimized(!isMinimized)}
            className="p-1.5 hover:bg-white/20 rounded-lg transition-colors"
            aria-label={isMinimized ? 'Maximize' : 'Minimize'}
          >
            {isMinimized ? <PlusIcon className="w-4 h-4" /> : <MinusIcon className="w-4 h-4" />}
          </button>
          <button
            onClick={onToggle}
            className="p-1.5 hover:bg-white/20 rounded-lg transition-colors"
            aria-label="Close chat"
          >
            <XMarkIcon className="w-4 h-4" />
          </button>
        </div>
      </div>

      {!isMinimized && (
        <>
          {/* Quick Actions */}
          <div className="p-3 bg-gray-50 border-b border-gray-200">
            <div className="flex space-x-2 overflow-x-auto scrollbar-hide">
              {quickActions.map((action, index) => (
                <button
                  key={index}
                  onClick={() => handleQuickAction(action)}
                  className={`flex items-center space-x-2 px-3 py-1.5 rounded-lg text-white text-sm whitespace-nowrap transition-all hover:scale-105 ${
                    connectionStatus === 'disconnected' ? 'bg-gray-300 cursor-not-allowed' : 'bg-blue-600'
                  }`}
                  disabled={isLoading || connectionStatus === 'disconnected'}
                >
                  <action.icon className="w-4 h-4 text-white" />
                  <span>{action.label}</span>
                </button>
              ))}
            </div>
          </div>

          {/* Messages */}
          <div className="flex-1 p-4 overflow-y-auto" style={{ height: 'calc(100% - 200px)' }}>
            <div className="space-y-4">
              {messages.map((message) => (
                <div
                  key={message.id}
                  className={`flex ${message.type === 'user' ? 'justify-end' : 'justify-start'} animate-fadeIn`}
                >
                  <div className={`flex max-w-[85%] ${message.type === 'user' ? 'flex-row-reverse' : 'flex-row'}`}>
                    <div className={`w-8 h-8 rounded-full flex items-center justify-center flex-shrink-0 ${
                      message.type === 'user' ? 'bg-gradient-to-br from-blue-500 to-blue-600 ml-2' : 'bg-gradient-to-br from-purple-500 to-purple-600 mr-2'
                    }`}>
                      {message.type === 'user' ? (
                        <UserIcon className="w-4 h-4 text-white" />
                      ) : (
                        <SparklesIcon className="w-4 h-4 text-white" />
                      )}
                    </div>
                    <div className={`px-4 py-3 rounded-2xl ${
                      message.type === 'user'
                        ? 'bg-blue-600 text-white'
                        : 'bg-gray-100 text-gray-800'
                    }`}>
                      {message.isStreaming ? (
                        <div className="flex space-x-1.5 py-1">
                          <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '0ms' }}></div>
                          <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '150ms' }}></div>
                          <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '300ms' }}></div>
                        </div>
                      ) : (
                        <>
                          <div className="prose prose-sm max-w-none">
                            <ReactMarkdown 
                              remarkPlugins={[remarkGfm]}
                              components={{
                                a: ({ children, ...props }) => (
                                  <a 
                                    {...props} 
                                    className="text-blue-600 hover:text-blue-800 underline"
                                    target="_blank"
                                    rel="noopener noreferrer"
                                  >
                                    {children}
                                  </a>
                                ),
                                code: ({ children, className, ...props }) => {
                                  const isInline = !className || !className.includes('language-');
                                  return isInline ? (
                                    <code className="bg-gray-200 px-1 py-0.5 rounded text-sm" {...props}>
                                      {children}
                                    </code>
                                  ) : (
                                    <code className="block bg-gray-900 text-gray-100 p-3 rounded-lg text-sm overflow-x-auto" {...props}>
                                      {children}
                                    </code>
                                  );
                                }
                              }}
                            >
                              {message.content}
                            </ReactMarkdown>
                          </div>
                          {message.metadata?.confidence && (
                            <div className="mt-2 pt-2 border-t border-gray-200 text-xs text-gray-500">
                              Confidence: {(message.metadata.confidence * 100).toFixed(0)}%
                            </div>
                          )}
                        </>
                      )}
                    </div>
                  </div>
                </div>
              ))}
              <div ref={messagesEndRef} />
            </div>
          </div>

          {/* Input */}
          <div className="p-4 border-t border-gray-200 bg-white rounded-b-2xl">
            <div className="flex items-center space-x-2">
              <input
                type="file"
                ref={fileInputRef}
                onChange={(e) => e.target.files && handleFileUpload(e.target.files)}
                className="hidden"
                accept=".pdf,.xlsx,.xls,.csv"
                multiple
              />
              <button
                onClick={() => fileInputRef.current?.click()}
                className="p-2 text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded-lg transition-colors"
                disabled={isLoading || connectionStatus === 'disconnected'}
                aria-label="Upload file"
              >
                <ClipboardDocumentListIcon className="w-5 h-5" />
              </button>
              <div className="flex-1 relative">
                <input
                  ref={inputRef}
                  type="text"
                  value={inputValue}
                  onChange={(e) => {
                    setInputValue(e.target.value);
                    // Send typing indicator via WebSocket
                    if (wsConnected && wsConnection) {
                      wsConnection.send(JSON.stringify({
                        type: 'typing',
                        typing: e.target.value.length > 0
                      }));
                    }
                  }}
                  onKeyPress={(e) => {
                    if (e.key === 'Enter' && !e.shiftKey) {
                      e.preventDefault();
                      handleSend();
                    }
                  }}
                  placeholder="Ask about samples, protocols, or lab procedures..."
                  className="w-full px-4 py-2.5 pr-10 border border-gray-300 rounded-xl focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent text-sm"
                  disabled={isLoading || connectionStatus === 'disconnected'}
                />
                {isLoading && (
                  <div className="absolute right-3 top-1/2 transform -translate-y-1/2">
                    <ArrowPathIcon className="w-4 h-4 text-gray-400 animate-spin" />
                  </div>
                )}
              </div>
              <button
                onClick={toggleVoiceInput}
                className={`p-2.5 rounded-lg transition-colors ${
                  listening 
                    ? 'bg-red-100 text-red-600 animate-pulse' 
                    : 'hover:bg-gray-200 text-gray-600'
                }`}
                disabled={isLoading || connectionStatus === 'disconnected'}
                aria-label={listening ? 'Stop recording' : 'Start voice input'}
              >
                <MicrophoneIcon className={`w-5 h-5 ${listening ? 'text-red-600' : 'text-gray-600'}`} />
              </button>
              <button
                data-send-button
                onClick={handleSend}
                disabled={!inputValue.trim() && uploadedFiles.current.length === 0 || isLoading || connectionStatus === 'disconnected'}
                className="p-2.5 bg-gradient-to-r from-blue-600 to-purple-600 text-white rounded-xl hover:from-blue-700 hover:to-purple-700 disabled:from-gray-300 disabled:to-gray-300 disabled:cursor-not-allowed transition-all shadow-sm"
                aria-label="Send message"
              >
                <PaperAirplaneIcon className="w-5 h-5" />
              </button>
            </div>
            
            {/* Footer */}
            <div className="flex justify-between items-center mt-3">
              <button
                onClick={() => {
                  setMessages([
                    {
                      id: '1',
                      content: "👋 Chat cleared! How can I help you with your laboratory work today?",
                      type: 'assistant',
                      timestamp: new Date(),
                    },
                  ]);
                  uploadedFiles.current = [];
                }}
                className="text-xs text-gray-500 hover:text-gray-700 transition-colors"
              >
                Clear chat
              </button>
              <div className="text-xs text-gray-400">
                Powered by RAG AI • TracSeq 2.0
              </div>
            </div>
          </div>
        </>
      )}
    </div>
  );
}; 
