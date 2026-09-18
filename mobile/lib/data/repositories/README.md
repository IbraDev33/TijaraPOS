# data/repositories/

The layer GetX controllers depend on. Decides online-vs-queued behavior
for writes (e.g. `SalesRepository.createSale` queues locally with an
idempotency key when offline) so that policy lives in one place instead of
being duplicated across modules.
