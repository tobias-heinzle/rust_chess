from chess import WHITE, BLACK

HEADER = '\033[95m'
OKBLUE = '\033[94m'
OKCYAN = '\033[96m'
OKGREEN = '\033[92m'
WARNING = '\033[93m'
FAIL = '\033[91m'
ENDC = '\033[0m'
BOLD = '\033[1m'
UNDERLINE = '\033[4m'

    
def print_board(board, engine_color = BLACK):

    board_str = str(board)

    new_board_str = "  -------------------\n"
    step = -1 if (engine_color == WHITE) else 1
    for line, number in zip(board_str.splitlines()[::step], range(1,9)[::-step]):
        new_line = ""
        for piece in line.split(" "):
            if piece == piece.lower():
                new_line += piece + " "
            else:
                new_line += BOLD + piece  + ENDC + " "

        new_board_str +=  str(number) + " | " + new_line.strip()[::step] + " |\n"
    new_board_str += "  -------------------\n"
    new_board_str += "    " + "A B C D E F G H"[::step]
    

    print(new_board_str)