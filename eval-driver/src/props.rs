extern crate anyhow;
use std::{collections::HashSet, sync::Arc};

use anyhow::{bail, Result};

use paralegal_policy::{
    assert_error, assert_warning,
    paralegal_spdg::{CallSite, DataSource, Identifier},
    Context, EdgeType, Marker, Node, NodeType, PolicyContext,
};

macro_rules! marker {
    ($id:ident) => {
        Marker::new_intern(stringify!($id))
    };
}

trait ContextExt {
    fn marked_nodes<'a>(&'a self, marker: Marker) -> Box<dyn Iterator<Item = Node<'a>> + 'a>;
}

impl ContextExt for PolicyContext {
    fn marked_nodes<'a>(&'a self, marker: Marker) -> Box<dyn Iterator<Item = Node<'a>> + 'a> {
        Box::new(
            self.desc()
                .controllers
                .keys()
                .copied()
                .flat_map(move |k| self.all_nodes_for_ctrl(k))
                .filter(move |node| self.has_marker(marker, *node)),
        )
    }
}

/// Asserts that there exists one controller which calls a deletion
/// function on every value (or an equivalent type) that is ever stored.
pub struct DeletionProp {
    cx: Arc<PolicyContext>,
}

impl DeletionProp {
    pub fn new(cx: Arc<PolicyContext>) -> Self {
        DeletionProp { cx }
    }

    pub fn check(self) -> Result<()> {
        // All types marked "sensitive"
        let types_to_check = self
            .cx
            .marked_type(marker!(sensitive))
            .filter(|t| {
                {
                    // If there is any controller
                    self.cx.desc().controllers.keys().any(|ctrl_id| {
                        // Where a source of that type
                        self.cx.srcs_with_type(*ctrl_id, *t).any(|sens_src| {
                            // Has data influence on
                            self.cx
                                .influencees(sens_src, paralegal_policy::EdgeType::Data)
                                .any(|influencee| {
                                    // A node with marker "influences"
                                    self.cx.has_marker(marker!(stores), influencee)
                                })
                        })
                    })
                }
            })
            // Mapped to their otype
            .flat_map(|t| self.cx.otypes(t))
            .collect::<Vec<_>>();
        let found_deleter = self.cx.desc().controllers.keys().any(|ctrl_id| {
            // For all types to check
            types_to_check.iter().all(|ty| {
                // If there is any src of that type
                self.cx.srcs_with_type(*ctrl_id, *ty).any(|node| {
                    // That has data flow influence on
                    self.cx
                        .influencees(node, paralegal_policy::EdgeType::Data)
                        // A node with marker "deletes"
                        .any(|influencee| self.cx.has_marker(marker!(deletes), influencee))
                })
            })
        });

        assert_error!(
            self.cx,
            found_deleter,
            "Did not find valid deleter for all types."
        );
        for ty in types_to_check {
            assert_error!(
                self.cx,
                found_deleter,
                format!("Type: {}", self.cx.describe_def(ty))
            )
        }

        Ok(())
    }

    pub fn check_lib(self) -> Result<()> {
        // All nodes marked sensitive that flow into a node marked stores
        let stored = self
            .cx
            .marked_nodes(marker!(sensitive))
            .filter(|sens| {
                self.cx
                    .influencees(*sens, EdgeType::Data)
                    .any(|influencee| self.cx.has_marker(marker!(stores), influencee))
            })
            .collect::<Vec<_>>();
        // Max deletions across all controllers
        let mut max_deleted = 0;
        for c_id in self.cx.desc().controllers.keys() {
            // All nodes marked from_storage that flow into a node marked deletes
            let deleted = self
                .cx
                .all_nodes_for_ctrl(*c_id)
                .filter(|n| {
                    self.cx.has_marker(marker!(from_storage), *n)
                        && self
                            .cx
                            .influencees(*n, EdgeType::Data)
                            .any(|influencee| self.cx.has_marker(marker!(deletes), influencee))
                })
                .count();
            max_deleted = std::cmp::max(max_deleted, deleted);
        }
        assert_error!(self.cx, max_deleted >= stored.len(), format!("Deleted less sensitive data than were stored in one controller. Stored {}, deleted {}", stored.len(), max_deleted));

        Ok(())
    }

    pub fn check_strict(self) -> Result<()> {
        // All types marked "sensitive"
        let types_to_check = self
            .cx
            .marked_type(marker!(sensitive))
            .filter(|t| {
                {
                    // If there is any controller
                    self.cx.desc().controllers.keys().any(|ctrl_id| {
                        // Where a source of that type
                        self.cx.srcs_with_type(*ctrl_id, *t).any(|sens_src| {
                            // Has data influence on
                            self.cx
                                .influencees(sens_src, paralegal_policy::EdgeType::Data)
                                .any(|influencee| {
                                    // A node with marker "influences"
                                    self.cx.has_marker(marker!(stores), influencee)
                                })
                        })
                    })
                }
            })
            // Mapped to their otype
            .flat_map(|t| self.cx.otypes(t))
            .collect::<Vec<_>>();

        let is_safe_noskip = |ctx: Arc<PolicyContext>, node: Node| -> bool {
            let safe_names = ["next", "into_iter", "deref_mut"];
            match node.typ.as_call_site() {
                Some(function) => safe_names
                    .into_iter()
                    .any(|name| ctx.desc().def_info[&function.function].name.as_str() == name),
                None => true,
            }
        };

        let flows_to_no_skip = |cx: Arc<PolicyContext>, src: Node, sink: Node| -> bool {
            if src == sink {
                return true;
            }

            let flow = &cx.desc().controllers.get(&src.ctrl_id).unwrap().data_flow.0;
            let mut queue = if let Some(source) = src.typ.as_data_source() {
                vec![source]
            } else {
                return false;
            };
            let mut seen = HashSet::<&CallSite>::new();

            while let Some(current) = queue.pop() {
                if let Some(source) = sink.typ.as_data_source() {
                    if source == current {
                        return true;
                    }
                }

                for next in flow.get(&current).into_iter().flatten() {
                    if let Some(sink) = sink.typ.as_data_sink() {
                        if next == sink {
                            return true;
                        }
                    }

                    if let Some((callsite, _)) = next.as_argument() {
                        let callsite_node = Node {
                            ctrl_id: src.ctrl_id,
                            typ: callsite.into(),
                        };

                        if is_safe_noskip(cx.clone(), callsite_node) && seen.insert(callsite) {
                            queue.push(DataSource::FunctionCall(callsite.clone()))
                        }
                    }
                }
            }

            false
        };

        let found_deleter = self.cx.desc().controllers.keys().any(|ctrl_id| {
            let auth_witnesses = self
                .cx
                .all_nodes_for_ctrl(*ctrl_id)
                .filter(|n| self.cx.has_marker(marker!(auth_witness), *n))
                .collect::<Vec<_>>();
            // Retrievers must be unconditional and have dataflow influence from an auth_witness
            let possible_retrievers = auth_witnesses
                .iter()
                .flat_map(|auth_witness| {
                    let cx_clone = self.cx.clone();
                    self.cx
                        .roots(*ctrl_id, EdgeType::Control)
                        .filter(move |r| cx_clone.flows_to(*auth_witness, *r, EdgeType::Data))
                })
                .collect::<Vec<_>>();

            // For all types to check
            types_to_check.iter().all(|ty| {
                let retrievers_for_type = possible_retrievers
                    .iter()
                    .filter(|retrieve| {
                        self.cx
                            .get_node_types(retrieve)
                            .is_some_and(|types| types.contains(ty))
                    })
                    .collect::<Vec<_>>();

                // A retriever of that type flows to delete without skipping
                retrievers_for_type.iter().any(|retriever| {
                    self.cx
                        .all_nodes_for_ctrl(*ctrl_id)
                        .filter(|node| self.cx.has_marker(marker!(deletes), *node))
                        .any(|delete| flows_to_no_skip(self.cx.clone(), **retriever, delete))
                })
            })
        });

        assert_error!(
            self.cx,
            found_deleter,
            "Did not find valid deleter for all types."
        );
        for ty in types_to_check {
            assert_error!(
                self.cx,
                found_deleter,
                format!("Type: {}", self.cx.describe_def(ty))
            )
        }

        Ok(())
    }
}

pub fn run_del_policy(ctx: Arc<Context>, version: &str) -> Result<()> {
    ctx.named_policy(Identifier::new_intern("Deletion"), |ctx| {
        let prop = DeletionProp::new(ctx);
        match version {
            "lib" => prop.check_lib(),
            "baseline" => prop.check(),
            "strict" => prop.check_strict(),
            other => bail!("version {} does not exist", other),
        }
    })
}

/// Storing data in the database must be associated to a user. This is
/// necessary for e.g. the deletion to work.
pub struct ScopedStorageProp {
    cx: Arc<PolicyContext>,
}

impl ScopedStorageProp {
    pub fn new(cx: Arc<PolicyContext>) -> Self {
        ScopedStorageProp { cx }
    }

    pub fn check(self) -> Result<()> {
        for c_id in self.cx.desc().controllers.keys() {
            let scopes = self
                .cx
                .all_nodes_for_ctrl(*c_id)
                .filter(|node| self.cx.has_marker(marker!(scopes_store), *node))
                .collect::<Vec<_>>();
            let stores = self
                .cx
                .all_nodes_for_ctrl(*c_id)
                .filter(|node| self.cx.has_marker(marker!(stores), *node))
                .collect::<Vec<_>>();
            let mut sensitives = self
                .cx
                .all_nodes_for_ctrl(*c_id)
                .filter(|node| self.cx.has_marker(marker!(sensitive), *node));

            let controller_valid = sensitives.all(|sens| {
                stores.iter().all(|&store| {
                    // sensitive flows to store implies some scope flows to store callsite
                    !(self
                        .cx
                        .flows_to(sens, store, paralegal_policy::EdgeType::Data))
                        ||
						// The sink that scope flows to may be another CallArgument attached to the store's CallSite, it doesn't need to be store itself.
						store.associated_call_site().is_some_and(|store_callsite| {
                            let found_scope = scopes.iter().any(|scope| {
                                self.cx.flows_to(
                                    *scope,
                                    store_callsite,
                                    paralegal_policy::EdgeType::Data,
                                ) &&
								self.cx.influencers(
									*scope,
									paralegal_policy::EdgeType::Data
								).any(|i| self.cx.has_marker(marker!(auth_witness), i))
                            });
                            assert_error!(
                                self.cx,
                                found_scope,
                                format!(
                                    "Stored sensitive isn't scoped. sensitive {} stored here: {}",
                                    self.cx.describe_node(sens),
                                    self.cx.describe_node(store)
                                )
                            );
                            found_scope
                        })
                })
            });

            assert_error!(
                self.cx,
                controller_valid,
                format!(
                    "Violation detected for controller: {}",
                    self.cx.describe_def(*c_id)
                ),
            );
        }
        Ok(())
    }

    pub fn check_lib(self) -> Result<()> {
        for c_id in self.cx.desc().controllers.keys() {
            let scopes = self
                .cx
                .all_nodes_for_ctrl(*c_id)
                .filter(|node| self.cx.has_marker(marker!(scopes_store), *node))
                .collect::<Vec<_>>();
            let stores = self
                .cx
                .all_nodes_for_ctrl(*c_id)
                .filter(|node| self.cx.has_marker(marker!(stores), *node))
                .collect::<Vec<_>>();
            let mut sensitives = self
                .cx
                .all_nodes_for_ctrl(*c_id)
                .filter(|node| self.cx.has_marker(marker!(sensitive), *node));

            let controller_valid = sensitives.all(|sens| {
                stores.iter().all(|&store| {
                    // sensitive flows to store implies some scope flows to store callsite
                    !(self
                        .cx
                        .flows_to(sens, store, paralegal_policy::EdgeType::Data))
                        ||
						// The sink that scope flows to may be another CallArgument attached to the store's CallSite, it doesn't need to be store itself.
						store.associated_call_site().is_some_and(|store_callsite| {
                            let found_scope = scopes.iter().any(|scope| {
                                self.cx.flows_to(
                                    *scope,
                                    store_callsite,
                                    paralegal_policy::EdgeType::Data,
                                ) &&
								self.cx.influencers(
									*scope,
									paralegal_policy::EdgeType::Data
								).any(|i| self.cx.has_marker(marker!(request_generated), i))
                            });
                            assert_error!(
                                self.cx,
                                found_scope,
                                format!(
                                    "Stored sensitive isn't scoped. sensitive {} stored here: {}",
                                    self.cx.describe_node(sens),
                                    self.cx.describe_node(store)
                                )
                            );
                            found_scope
                        })
                })
            });

            assert_error!(
                self.cx,
                controller_valid,
                format!(
                    "Violation detected for controller: {}",
                    self.cx.describe_def(*c_id)
                ),
            );
        }
        Ok(())
    }

    pub fn check_strict(self) -> Result<()> {
        self.check()
    }
}

pub fn run_sc_policy(ctx: Arc<Context>, version: &str) -> Result<()> {
    ctx.named_policy(Identifier::new_intern("Scoped Storage"), |ctx| {
        let prop = ScopedStorageProp::new(ctx);
        match version {
            "lib" => prop.check_lib(),
            "baseline" => prop.check(),
            "strict" => prop.check_strict(),
            other => bail!("version {} does not exist", other),
        }
    })
}

/// If sensitive data is released, the release must be scoped, and all inputs to the scope must be safe.
pub struct AuthDisclosureProp {
    cx: Arc<PolicyContext>,
}

impl AuthDisclosureProp {
    pub fn new(cx: Arc<PolicyContext>) -> Self {
        AuthDisclosureProp { cx }
    }

    // TODO: differentiate between different kinds of sinks - email sinks vs print sinks for all annotation levels
    pub fn check(self) -> Result<()> {
        let mut sens_to_sink = 0;
        for c_id in self.cx.desc().controllers.keys() {
            // All srcs that have no influencers
            let roots = self
                .cx
                .roots(*c_id, paralegal_policy::EdgeType::Data)
                .collect::<Vec<_>>();

            let safe_scopes = self
                .cx
                // All nodes marked "safe"
                .all_nodes_for_ctrl(*c_id)
                .filter(|n| self.cx.has_marker(marker!(safe_source), *n))
                // And all nodes marked "safe_with_bless"
                .chain(self.cx.all_nodes_for_ctrl(*c_id).filter(|node| {
                    self.cx.has_marker(marker!(safe_source_with_bless), *node)
                        && self
                            .cx
                            // That are influenced by a node marked "bless"
                            .influencers(*node, paralegal_policy::EdgeType::DataAndControl)
                            .any(|b| self.cx.has_marker(marker!(bless_safe_source), b))
                }))
                .collect::<Vec<_>>();
            let sinks = self
                .cx
                .all_nodes_for_ctrl(*c_id)
                .filter(|n| self.cx.has_marker(marker!(sink), *n))
                .collect::<Vec<_>>();
            let sensitives = self
                .cx
                .all_nodes_for_ctrl(*c_id)
                .filter(|node| self.cx.has_marker(marker!(sensitive), *node));

            for sens in sensitives {
                for sink in sinks.iter() {
                    // sensitive flows to store implies
                    if !self
                        .cx
                        .flows_to(sens, *sink, paralegal_policy::EdgeType::Data)
                    {
                        continue;
                    }
                    sens_to_sink += 1;

                    let Some(sink_callsite) = sink.associated_call_site() else {
                        assert_error!(
                            self.cx,
                            false,
                            format!(
                                "sink {} does not have associated callsite",
                                self.cx.describe_node(*sink)
                            )
                        );
                        continue;
                    };

                    // scopes for the store
                    let store_scopes = self
                        .cx
                        .influencers(sink_callsite, paralegal_policy::EdgeType::Data)
                        .filter(|n| self.cx.has_marker(marker!(scopes), *n))
                        .collect::<Vec<_>>();
                    assert_error!(
                        self.cx,
                        !store_scopes.is_empty(),
                        format!(
                            "Did not find any scopes for sink {}",
                            self.cx.describe_node(*sink)
                        )
                    );

                    // all flows are safe before scope
                    let safe_before_scope = self.cx.always_happens_before(
                        roots.iter().cloned(),
                        |n| safe_scopes.contains(&n),
                        |n| store_scopes.contains(&n),
                    )?;

                    assert_error!(
                        self.cx,
                        safe_before_scope.holds(),
                        format!(
                            "Sensitive {} flowed to sink {} which did not have safe scopes",
                            self.cx.describe_node(sens),
                            self.cx.describe_node(*sink),
                        )
                    );
                    safe_before_scope.report(self.cx.clone());
                }
            }
        }
        assert_warning!(
            self.cx,
            sens_to_sink != 0,
            "No sensitives flowed to any sinks across all controllers. Property is vacuous."
        );
        Ok(())
    }

    pub fn check_lib(self) -> Result<()> {
        let mut sens_to_sink = 0;
        for c_id in self.cx.desc().controllers.keys() {
            // All srcs that have no influencers
            let roots = self
                .cx
                .roots(*c_id, paralegal_policy::EdgeType::Data)
                .collect::<Vec<_>>();

            let safe_scopes = self
                .cx
                // All nodes marked "safe", "request_generated", "server_state", "from_storage"
                .all_nodes_for_ctrl(*c_id)
                .filter(|n| {
                    self.cx.has_marker(marker!(safe_source), *n)
                        || self.cx.has_marker(marker!(request_generated), *n)
                        || self.cx.has_marker(marker!(server_state), *n)
                        || self.cx.has_marker(marker!(from_storage), *n)
                })
                .collect::<Vec<_>>();
            let sinks = self
                .cx
                .all_nodes_for_ctrl(*c_id)
                .filter(|n| self.cx.has_marker(marker!(sink), *n))
                .collect::<Vec<_>>();

            let sensitives_and_from_storage = self
                .cx
                .all_nodes_for_ctrl(*c_id)
                .filter(|node| self.cx.has_marker(marker!(sensitive), *node))
                .chain(
                    self.cx
                        .all_nodes_for_ctrl(*c_id)
                        .filter(|node| self.cx.has_marker(marker!(from_storage), *node)),
                );

            for sens in sensitives_and_from_storage {
                for sink in sinks.iter() {
                    // sensitive flows to store implies
                    if !self
                        .cx
                        .flows_to(sens, *sink, paralegal_policy::EdgeType::Data)
                    {
                        continue;
                    }
                    sens_to_sink += 1;

                    // scopes for the store
                    let store_scopes = if let Some(sink_callsite) = sink.associated_call_site() {
                        // If the store is a callsite, it is any `scopes` that flows into it.
                        self.cx
                            .influencers(sink_callsite, paralegal_policy::EdgeType::Data)
                            .filter(|n| self.cx.has_marker(marker!(scopes), *n))
                            .collect::<Vec<_>>()
                    } else if let Node {
                        ctrl_id,
                        typ: NodeType::Return(_),
                    } = sink
                    {
                        // If the store is the Controller Return, it is anything marked `request_generated`.
                        self.cx
                            .all_nodes_for_ctrl(*ctrl_id)
                            .filter(|n| self.cx.has_marker(marker!(request_generated), *n))
                            .collect::<Vec<_>>()
                    } else {
                        assert_error!(
                            self.cx,
                            false,
                            format!(
                                "sink {} does not have associated callsite",
                                self.cx.describe_node(*sink)
                            )
                        );
                        continue;
                    };

                    assert_error!(
                        self.cx,
                        !store_scopes.is_empty(),
                        format!(
                            "Did not find any scopes for sink {}",
                            self.cx.describe_node(*sink)
                        )
                    );

                    // all flows are safe before scope
                    let safe_before_scope = self.cx.always_happens_before(
                        roots.iter().cloned(),
                        |n| safe_scopes.contains(&n),
                        |n| store_scopes.contains(&n),
                    )?;

                    assert_error!(
                        self.cx,
                        safe_before_scope.holds(),
                        format!(
                            "Sensitive {} flowed to sink {} which did not have safe scopes",
                            self.cx.describe_node(sens),
                            self.cx.describe_node(*sink),
                        )
                    );
                    safe_before_scope.report(self.cx.clone());
                }
            }
        }
        assert_warning!(
            self.cx,
            sens_to_sink != 0,
            "No sensitives flowed to any sinks across all controllers. Property is vacuous."
        );
        Ok(())
    }

    pub fn check_strict(self) -> Result<()> {
        self.check()
    }
}

pub fn run_dis_policy(ctx: Arc<Context>, version: &str) -> Result<()> {
    ctx.named_policy(Identifier::new_intern("Authorized Disclosure"), |ctx| {
        let prop = AuthDisclosureProp::new(ctx);
        match version {
            "lib" => prop.check_lib(),
            "baseline" => prop.check(),
            "strict" => prop.check_strict(),
            other => bail!("version {} does not exist", other),
        }
    })
}
