let io = import "io.ncl" in

io.bind
  (io.print_line "What is your name?")
  (fun _ =>
    io.bind
      io.read_line
      (fun name =>
        io.print_line ("Hello, " ++ name ++ "!")))
