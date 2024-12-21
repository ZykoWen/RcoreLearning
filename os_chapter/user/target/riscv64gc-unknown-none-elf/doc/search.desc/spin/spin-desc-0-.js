searchState.loadedDescShard("spin", 0, "This crate provides spin-based versions of the primitives …\nSynchronization primitive allowing multiple threads to …\nSynchronization primitives for lazy evaluation.\nLocks that have the same behaviour as a mutex.\nSynchronization primitives for one-time evaluation.\nA lock that provides data access to either one writer or …\nA primitive that synchronizes the execution of multiple …\nA <code>BarrierWaitResult</code> is returned by <code>wait</code> when all threads …\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nReturns whether this thread from <code>wait</code> is the “leader …\nCreates a new barrier that can block a given number of …\nBlocks the current thread until all threads have …\nA value which is initialized on the first access.\nCreates a new lazy value using <code>Default</code> as the initializing …\nForces the evaluation of this lazy value and returns a …\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCreates a new lazy value with the given initializing …\nA spin-based lock providing mutually exclusive access to …\nA generic guard that will protect some data access and …\nA spin lock providing mutually exclusive access to data.\nA guard that provides mutable data access.\nA spin-based ticket lock providing mutually exclusive …\nA guard that protects some data.\nThe dropping of the MutexGuard will release the lock it …\nForce unlock this <code>SpinMutex</code>.\nForce unlock this <code>TicketMutex</code>, by serving the next ticket.\nForce unlock this <code>Mutex</code>.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns a mutable reference to the underlying data.\nReturns a mutable reference to the underlying data.\nReturns a mutable reference to the underlying data.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nConsumes this <code>SpinMutex</code> and unwraps the underlying data.\nConsumes this <code>TicketMutex</code> and unwraps the underlying data.\nConsumes this <code>Mutex</code> and unwraps the underlying data.\nReturns <code>true</code> if the lock is currently held.\nReturns <code>true</code> if the lock is currently held.\nReturns <code>true</code> if the lock is currently held.\nLeak the lock guard, yielding a mutable reference to the …\nLeak the lock guard, yielding a mutable reference to the …\nLeak the lock guard, yielding a mutable reference to the …\nLocks the <code>SpinMutex</code> and returns a guard that permits …\nLocks the <code>TicketMutex</code> and returns a guard that permits …\nLocks the <code>Mutex</code> and returns a guard that permits access to …\nCreates a new <code>SpinMutex</code> wrapping the supplied data.\nCreates a new <code>TicketMutex</code> wrapping the supplied data.\nCreates a new <code>Mutex</code> wrapping the supplied data.\nTry to lock this <code>SpinMutex</code>, returning a lock guard if …\nTry to lock this <code>TicketMutex</code>, returning a lock guard if …\nTry to lock this <code>Mutex</code>, returning a lock guard if …\nInitialization constant of <code>Once</code>.\nA primitive that provides lazy one-time initialization.\nPerforms an initialization routine once and only once. The …\nReturns the argument unchanged.\nReturns a reference to the inner value if the <code>Once</code> has …\nReturns a mutable reference to the inner value if the <code>Once</code> …\nCreates a new initialized <code>Once</code>.\nCalls <code>U::from(self)</code>.\nReturns a reference to the inner value if the <code>Once</code> has …\nCreates a new <code>Once</code>.\nLike <code>Once::get</code>, but will spin if the <code>Once</code> is in the …\nReturns a the inner value if the <code>Once</code> has been initialized.\nSpins until the <code>Once</code> contains a value.\nA lock that provides data access to either one writer or …\nA guard that provides immutable data access.\nA guard that provides immutable data access but can be …\nA guard that provides mutable data access.\nDowngrades the writable lock guard to a readable, shared …\nDowngrades the upgradeable lock guard to a readable, …\nDowngrades the writable lock guard to an upgradable, …\nForce decrement the reader count.\nForce unlock exclusive write access.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns a mutable reference to the underlying data.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nConsumes this <code>RwLock</code>, returning the underlying data.\nLeak the lock guard, yielding a reference to the …\nLeak the lock guard, yielding a mutable reference to the …\nLeak the lock guard, yielding a reference to the …\nCreates a new spinlock wrapping the supplied data.\nLocks this rwlock with shared read access, blocking the …\nReturn the number of readers that currently hold the lock …\nAttempt to acquire this lock with shared read access.\nTries to upgrade an upgradeable lock guard to a writable …\nTries to obtain an upgradeable lock guard.\nAttempt to lock this rwlock with exclusive write access.\nUpgrades an upgradeable lock guard to a writable lock …\nObtain a readable lock guard that can later be upgraded to …\nLock this rwlock with exclusive write access, blocking the …\nReturn the number of writers that currently hold the lock.")