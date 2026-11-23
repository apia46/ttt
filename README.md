# Infinitely* Recursive Tic Tac Toe
A natural extension to the rules of https://en.wikipedia.org/wiki/Ultimate_tic-tac-toe, to an arbitrary number of layers.<p>
My second attempt at doing this. See https://apia46.itch.io/tictactoe for the first.
## Controls
* LMB - interact
* RMB/MMB drag or WASD - move camera
* MMB scroll - zoom camera
* H - return camera to starting position
## What?
The game has a recursive structure, such that a layer N board contains 9 layer N-1 boards.
Players take turns making layer 0 moves, though they are only allowed to choose a move within the domain.
A player making a row of 3 of layer K pieces within a layer K+1 board generates a layer K+1 move to occur at that board, for that player.
Thus, each turn creates some number of moves starting in layer 0, depending on how many higher level rows of 3 were subsequently created.<p>
A domain is specified as some specific board (let it be of layer M), and thus consists of a cell (1 through 9) for each layer N down to M+1, if the game has N layers in total.
The domain starts as the centermost layer 1 board, and a layer K move at some cell C in the layer K+1 board causes the layer K+2 cell of the domain to be changed to C.
For example, a layer 0 move in the middle-left cell of some layer 1 board causes the domain to move to the middle-left layer 1 board in typically the same layer 2 board, unless a layer 1 move also occured. 
This is with the exception of a layer N-1 move in a N-layered game, which cannot change the Nth cell of the domain position, unless the game board is expanded mid-game.<p>
If the domain is moved to somewhere that is filled, it will temporarily expand by layer until there is a valid move.
A layer K piece counts as filling the layer K board, and all boards within it.
A board also counts as filled if it is tied (all 9 sub-boards are filled, with no winner).
You cannot move in a filled board, even if it is within the domain and there is some empty layer 0 cell within it.<p>
An N-layered game ends when either player makes a layer N move, or if the layer N board is tied.

<img width="1834" height="1024" alt="explanation" src="https://github.com/user-attachments/assets/93403b46-a766-4cb7-a5b4-b2258782ac5c" />
