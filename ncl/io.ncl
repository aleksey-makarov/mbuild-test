# IO monad library for Nickel
# IO actions are represented as enum variants (free monad style).
# A Rust interpreter walks the tree and performs actual effects.

{
  # -- Monad core --

  return = fun x => 'Return x,

  bind = fun a k => 'Bind { action = a, cont = k },

  seq = fun a b => 'Bind { action = a, cont = fun _x => b },

  map = fun f a => 'Bind { action = a, cont = fun x => 'Return (f x) },

  # -- Primitives --

  print_line = fun s => 'PrintLine s,

  print = fun s => 'Print s,

  read_line = 'ReadLine null,

  read_file = fun p => 'ReadFile p,

  write_file = fun p c => 'WriteFile { path = p, content = c },

  get_env = fun name => 'GetEnv name,

  args = 'Args null,

  # -- Combinators --

  sequence = fun actions =>
    std.array.fold_right
      (fun a acc =>
        'Bind {
          action = a,
          cont = fun _result => acc,
        })
      ('Return null)
      actions,

  for_each = fun items f =>
    std.array.fold_right
      (fun item acc =>
        'Bind {
          action = f item,
          cont = fun _result => acc,
        })
      ('Return null)
      items,
}
