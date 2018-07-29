# Object Graph

Things are starting to get a little complex, which for the moment is fine, but
I need a way to keep things straight so I'm making myself a little cheat-sheet.


 + `System`
   + __Owns__: Thread (`JoinHandle`)
   + __Owns__: `Scheduler`
   + __Owns__: Configuration

 + `Scheduler`
   + __Owns__: Cell

 + `Cell`
   + __Owns__: `Actor`
   + __Owns__: `Actor` Producer
   + __Owns__: Send/Receive Channels
   + __Links__: Parent `Scheduler`
 
 + `Address`
   + __Links__: `Cell`
   + __Links__: Send Channel

 + `Context`
   + __Links__: Parent `Scheduler`
   + __Contains__: UUID of Parent `Cell`

Things are _mostly_ top-down with the exception that the cell links back up to the
scheduler. There are some minor details in the code about how this all relates that
I won't include here since it's too in flux.

For anyone stumbling across this, I do intend to make sure the final object-heirarchy
is nice and clean, but that will come much later. When I'm not writing dirty hacks
just to test my initial ideas.
