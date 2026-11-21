import { installGlobalDelegates, installInstanceApi } from "./instance.ts";

// W4: the handle tables are PER INSTANCE now. `installGlobalDelegates()`
// defines these names on `globalThis` as accessors that forward to the
// current instance, so the call sites below/elsewhere stay unchanged and
// no mutable handle state lives on `globalThis`.
installGlobalDelegates();
installInstanceApi();

// Install the per-instance handle-table delegates (W4)
