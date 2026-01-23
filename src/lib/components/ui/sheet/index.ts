import { Dialog as SheetPrimitive } from "bits-ui";

import Sheet from "./sheet.svelte";
import SheetContent, {
  sheetVariants,
  type SheetSide,
} from "./sheet-content.svelte";
import SheetHeader from "./sheet-header.svelte";
import SheetFooter from "./sheet-footer.svelte";
import SheetTitle from "./sheet-title.svelte";
import SheetDescription from "./sheet-description.svelte";

const SheetTrigger = SheetPrimitive.Trigger;
const SheetClose = SheetPrimitive.Close;
const SheetPortal = SheetPrimitive.Portal;
const SheetOverlay = SheetPrimitive.Overlay;

export {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetFooter,
  SheetTitle,
  SheetDescription,
  SheetTrigger,
  SheetClose,
  SheetPortal,
  SheetOverlay,
  sheetVariants,
  type SheetSide,
  // Aliases
  Sheet as Root,
  SheetContent as Content,
  SheetHeader as Header,
  SheetFooter as Footer,
  SheetTitle as Title,
  SheetDescription as Description,
  SheetTrigger as Trigger,
  SheetClose as Close,
  SheetPortal as Portal,
  SheetOverlay as Overlay,
};
