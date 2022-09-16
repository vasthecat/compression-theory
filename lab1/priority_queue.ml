type 'a queue = Empty | Node of int * 'a * 'a queue * 'a queue
let is_single = function
    | Node (_, _, Empty, Empty) -> true
    | _ -> false
let rec insert queue priority value =
    match queue with
    | Empty -> Node (priority, value, Empty, Empty)
    | Node (priority', value', left, right) ->
        if priority <= priority'
        then Node (priority, value, insert right priority' value', left)
        else Node (priority', value', insert right priority value, left)
let rec remove_top = function
    | Empty -> None
    | Node (priority, value, left, Empty) -> Some left
    | Node (priority, value, Empty, right) -> Some right
    | Node (priority, value,
            (Node (lprio, lelt, _, _) as left),
            (Node (rprio, relt, _, _) as right)) ->
        if lprio <= rprio
        then let branch = match remove_top left with
               | None -> Empty
               | Some v -> v
             in Some (Node (lprio, lelt, branch, right))
        else let branch = match remove_top right with
               | None -> Empty
               | Some v -> v
             in Some (Node (rprio, relt, left, branch))
let extract = function
    | Empty -> None
    | Node (priority, value, _, _) as queue ->
        match remove_top queue with
        | None -> Some (priority, value, Empty)
        | Some node -> Some (priority, value, node)

