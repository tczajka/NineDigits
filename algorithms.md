#  Sudoku solver

One part of the engine is a regular, single-player Sudoku solver: given an unfinished Sudoku board, generate full solutions. We could just use an existing solver, but it's more fun to implement one from scratch. Our solver is loosely based on ideas of [Tdoku](https://t-dillon.github.io/tdoku/) however.

The solver is used in two ways by the game engine:
* To see if a board is still solvable and thus to generate legal moves.
* To exhaustively generate all solutions in the endgame.

We define a set of 0-1 variables in each 3x3 box. Boxes are indexed by (i, j):
* a<sub>i,j,y,x,d</sub> = 1 if the cell at (y, x) of the box contains the digit d
* h<sub>i,j,y,d</sub> = 1 if the mini-row consisting of three cells in row y of the box does **not** contain the digit d
* v<sub>i,j,x,d</sub> = 1 if the mini-column consisting of three cells in column x of the box does **not** contain the digit d

To solve a Sudoku, we are trying to solve a set of equations:
* Each cell contains one digit: <br>
a<sub>i,j,y,x,1</sub> + a<sub>i,j,y,x,2</sub> + ... + a<sub>i,j,y,x,9</sub> = 1
* Definition of h: <br>
a<sub>i,j,y,0,d</sub> + a<sub>i,j,y,1,d</sub> + a<sub>i,j,y,2,d</sub> + h<sub>i,j,y,d</sub> = 1
* Definition of v: <br>
a<sub>i,j,0,x,d</sub> + a<sub>i,j,1,x,d</sub> + a<sub>i,j,2,x,d</sub> + v<sub>i,j,x,d</sub> = 1
* Each row must contain a digit once:<br>
h<sub>i,0,y,d</sub> + h<sub>i,1,y,d</sub> + h<sub>i,2,y,d</sub> = 2
* Each column must contain a digit once:<br>
v<sub>0,j,x,d</sub> + v<sub>1,j,x,d</sub> + v<sub>2,j,x,d</sub> = 2
* Each box must contain a digit once. The following two equations are redundant, but we include both:<br>
h<sub>i,j,0,d</sub> + h<sub>i,j,1,d</sub> + h<sub>i,j,2,d</sub> = 2<br>
v<sub>i,j,0,d</sub> + v<sub>i,j,1,d</sub> + v<sub>i,j,2,d</sub> = 2

Each equation is simply a sum over certain variables. We solve this using a variant of [DPLL](https://en.wikipedia.org/wiki/DPLL_algorithm) specialized to such sums. In particular:
* Whenever enough variables in an equation are set to 1, we know to set the other ones to 0.
* Whenever enough variables in an equation are set to 0, we know to set the other ones to 1.
* If we can't make progress, branch on a variable and consider both cases in turn.

We always branch on h or v variables. Our heuristic is:
* Pick a band (3 boxes in a row or column) with the smallest number of undecided h and v variables.
* Pick a digit with the smallest number of undecided variables in that band.
* Branch on any undecided h or v variable in the box with that digit.

The solver uses SIMD instructions to operate on all variables in a box in parallel, and all h or v variables in a band in parallel. There are 135 variables per box, we pack them all in a 256-bit SIMD register.

We simply use depth-first backtracking after branching. When looking for just one solution, a different strategy might be better. With depth-first search we could get stuck in an impossible subtree while there might be easy solutions in other subtrees. That's a future improvement to consider.

# Legal move generation

To generate all legal moves in a position, we generate the set of possible digits in each cell. To do this, start with empty sets and iterate:
* Pick any cell where the set hasn't yet been fully determined.
* Use the solver to try to generate an arbitrary solution with a digit in that cell outside of the set seen so far.
* If successful: include the digits from that solution in the sets.
* If unsuccessful: the set of digits in the cell we picked has now been fully determined.

Almost always we can generate all legal moves within milliseconds. But in some rare cases it takes much longer. In those cases we put a timeout and have to make do with the valid moves found until then.

But we always need to have found at least 2 solutions in order to have any legal moves at all. So we could potentially time out just trying to find any legal move.

# Opening

Before there are 15 digits on the board we simply generate all legal moves and pick a random one. That's it. At least this makes us unpredictable!

# Middle game

The middle game is when we are out of the opening, but have not yet exhaustively generated all the solutions.

What we do is try to generate up to 100,000 solutions. Almost always this is rather fast, but we have a timeout just in case. If we manage to generate all the solutions, we switch to the endgame.

Next we generate all legal moves. This may sometimes involve finding a few more solutions with digits that have not been seen in the 100,000 solutions generated so far.

Now pick the move that leaves as many of those solutions as possible. We are not trying to make a winning move, we are just trying to make the game hard for the opponent.

If this leaves more than 90,000 solutions we simply make the move. Otherwise, don't make the move just yet. It's our provisional choice.

Now we again try to exhaustively generate all the solutions that will remain after our provisional move. If that succeeds, we try to analyze the resulting endgame.

If the resulting endgame is winning for the opponent, we enter **panic mode**. We throw away the provisional move, pick a different move, and repeat the whole process. Eventually we find a move that isn't provably losing, as far as we know, and make that one.

In theory we could find out that all moves are losing, already in the middle game. I have never seen it happen. But if it did happen, we would go back to our first provisional move and hope for the best.

# Endgame

In the endgame we have all the solutions. Now we throw away the original board representation and simply operate on sets of possible solutions. We no longer have to think about board geometry, we just have possible sequences of digits in the 81 cells.

Looking at solution sets helps us recognize equivalent board states easily. For instance: if a 2 in cell A implies there is a 3 in cell B, and vice-versa, both moves will lead to the same set of solutions. So we will automatically treat them as equivalent moves.

When we generate solutions in our solver, we associate with each solution a random 64-bit identifier. Now we define a hash function for a set of solutions: it's simply the **xor** of all the identifiers in the set. This makes it a [universal hash function](https://sortingsearching.com/2020/05/21/hashing.html).

We try to solve an endgame position by depth-first search. In particular, we do the following:
* Using a single scan over the solutions, compute for each move two things:
  * the number of remaining solutions after the move
  * the hash of the resulting solution set
* Then we do a quick check to see whether any move is immediately winning in one move, or whether we have already solved the position after the move and we know it's losing.
* If not, we **compress** the solution set by throwing away those cells that have only one possible digit. Moves in that cell are not allowed any more. So we end up with fewer than 81 cells that are still relevant.
* Then we sort the valid moves by the number of remaining solutions.
* For each move, starting with the simplest remaining set, we generate a new, filtered solution set after that move, and try to solve it recursively.

One interesting optimization I found is that if we consider a move A first and it loses to a response B, then we do not have to consider the move B at all because it loses to the response A by transposition. This relies on the fact that we consider moves in order of simplicity: it implies that the response A is still going to be a legal move after B.

At the top level of our search there is a slight modification: if we are running out of time then instead of looking at the least complicated moves with the smallest number of remaining solutions, we do the opposite: we start looking at the **most** complicated moves with the largest number of remaining solutions. The idea is that if we run out of allocated time, we are going to play the last move that we couldn't solve. We are hoping that one of two things will happen:
* maybe there is no good response and the resulting position is actually losing for the opponent; which is perhaps why we couldn't solve the position -- winning positions are often quick to solve, losing positions are more difficult
* or if it does have a good response then hopefully the opponent won't find it either!

# Summary

The game seems to be of the sort where until we can prove which side has a winning position, it is hard to say who has the advantage. This is why we simply play random moves in the opening: we have no idea what we're trying to do! All the intelligence is in trying to solve endgames, and lacking that, in trying to complicate the position so that the opponent can't figure it out either.

I've thought about the idea of trying to estimate winning chances somehow even before solving endgames. It might be possible to some extent. We know certain features of positions. We know how many squares are left, how many digits are possible in each cell, and can count (or estimate) how many solutions there are in total. But I have never implemented that.
