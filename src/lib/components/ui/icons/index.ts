// Re-export commonly used icons from lucide-svelte for consistency
// This provides a centralized place to manage icon usage across the app

export {
  // View icons
  LayoutGrid,
  CheckCircle,
  History,
  MessageSquare,

  // Panel toggle icons
  BarChart2,
  Wrench,
  FileText,
  CheckSquare,
  ListTodo,

  // Action icons
  Trash2,
  Square,
  Send,
  X,
  Plus,
  Minus,
  RefreshCw,
  Download,
  Upload,
  Copy,
  Check,

  // Status icons
  AlertCircle,
  AlertTriangle,
  Info,
  HelpCircle,
  Loader2,

  // Navigation icons
  ChevronDown,
  ChevronUp,
  ChevronLeft,
  ChevronRight,
  ArrowLeft,
  ArrowRight,

  // Content icons
  Folder,
  FolderOpen,
  File,
  Image,
  Code,
  Terminal,
  Settings,
  Search,
  Filter,
  Eye,
  EyeOff,

  // Social icons
  Github,
  ExternalLink,

  // Voice icons
  Mic,
  MicOff,
} from "lucide-svelte";

// Icon size presets for consistency
export const ICON_SIZES = {
  xs: 14,
  sm: 16,
  md: 20,
  lg: 24,
  xl: 32,
} as const;

export type IconSize = keyof typeof ICON_SIZES;
