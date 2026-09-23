import GooseLogo from './GooseLogo';
import BreathingLeaf from './BreathingLeaf';
import { ChatState } from '../types/chatState';
import { defineMessages, useIntl } from '../i18n';

interface LoadingGooseProps {
  message?: string;
  chatState?: ChatState;
}

const i18n = defineMessages({
  loadingConversation: {
    id: 'loadingGoose.loadingConversation',
    defaultMessage: 'loading conversation...',
  },
  thinking: {
    id: 'loadingGoose.thinking',
    defaultMessage: 'ElRoi is thinking...',
  },
  streaming: {
    id: 'loadingGoose.streaming',
    defaultMessage: 'ElRoi is working on it...',
  },
  waiting: {
    id: 'loadingGoose.waiting',
    defaultMessage: 'ElRoi is waiting...',
  },
  compacting: {
    id: 'loadingGoose.compacting',
    defaultMessage: 'ElRoi is compacting the conversation...',
  },
  idle: {
    id: 'loadingGoose.idle',
    defaultMessage: 'ElRoi is working on it...',
  },
  restartingAgent: {
    id: 'loadingGoose.restartingAgent',
    defaultMessage: 'restarting session...',
  },
});

/* Every busy state shares one indicator. Swapping marks mid-wait only drew
   attention to the machinery; the work is the same work whichever phase it is
   in, so the leaf just keeps breathing until ElRoi has something to say. */
const BUSY_INDICATOR = <BreathingLeaf />;

const STATE_ICONS: Record<ChatState, React.ReactNode> = {
  [ChatState.LoadingConversation]: BUSY_INDICATOR,
  [ChatState.Thinking]: BUSY_INDICATOR,
  [ChatState.Streaming]: BUSY_INDICATOR,
  [ChatState.WaitingForUserInput]: BUSY_INDICATOR,
  [ChatState.Compacting]: BUSY_INDICATOR,
  [ChatState.Idle]: <GooseLogo size="small" hover={false} />,
  [ChatState.RestartingAgent]: BUSY_INDICATOR,
};

const STATE_MESSAGE_KEYS: Record<ChatState, keyof typeof i18n> = {
  [ChatState.LoadingConversation]: 'loadingConversation',
  [ChatState.Thinking]: 'thinking',
  [ChatState.Streaming]: 'streaming',
  [ChatState.WaitingForUserInput]: 'waiting',
  [ChatState.Compacting]: 'compacting',
  [ChatState.Idle]: 'idle',
  [ChatState.RestartingAgent]: 'restartingAgent',
};

const LoadingGoose = ({ message, chatState = ChatState.Idle }: LoadingGooseProps) => {
  const intl = useIntl();
  const displayMessage = message || intl.formatMessage(i18n[STATE_MESSAGE_KEYS[chatState]]);
  const icon = STATE_ICONS[chatState];

  return (
    <div className="w-full animate-fade-slide-up">
      <div
        data-testid="loading-indicator"
        className="flex items-center gap-2 text-xs text-text-primary py-2"
      >
        {icon}
        {displayMessage}
      </div>
    </div>
  );
};

export default LoadingGoose;
