"""Framework-free policy fixture; image decoding and host texture ownership are external."""
class Cache:
    def __init__(self):
        self.namespace = None
        self.epoch = 0
        self.resident = {}
        self.pending = set()

    def change_context(self, namespace, generation):
        context = (namespace,generation)
        if getattr(self,"context",None) == context:
            return
        same = self.namespace == namespace
        self.epoch += 1
        self.pending.clear()
        self.resident = {key:value for key,value in self.resident.items() if same and value[0] == "image" and value[1] is not None}
        self.namespace, self.context = namespace, context

    def request(self, item, edge, revision):
        key = (item,edge,revision)
        if key not in self.resident:
            self.pending.add((key,self.epoch))
        return key, self.epoch

    def complete(self, ticket, kind, handle):
        key, epoch = ticket
        if epoch != self.epoch or ticket not in self.pending:
            return False
        self.pending.remove(ticket)
        self.resident[key] = (kind,handle)
        return True
