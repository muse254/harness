export const idlFactory = ({ IDL }) => {
  return IDL.Service({
    'get_program_code' : IDL.Func([], [IDL.Vec(IDL.Nat8)], ['query']),
    'hello' : IDL.Func([IDL.Text], [IDL.Text], []),
    'register_device' : IDL.Func([IDL.Text], [], []),
  });
};
export const init = ({ IDL }) => { return []; };
