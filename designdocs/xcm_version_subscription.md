```mermaid
sequenceDiagram
    participant Frequency
    participant Westend
    participant AssetHub

    Note over Frequency: Event::VersionNotifyRequested
    Frequency->>Westend: SubscribeVersion
    Westend-->>AssetHub: Horizontal messages processed by AssetHub
    Note over AssetHub: Event::VersionNotifyStarted
    AssetHub-->>Westend: QueryResponse
    Westend-->>Frequency: Horizontal messages processed by Frequency
```
