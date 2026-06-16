Feature: BYOK Virtual Key Management
  As a gateway administrator
  I want to manage virtual API keys with rate limits and budgets
  So that teams can access LLM providers through controlled gateway keys

  Background:
    Given the gateway is running with virtual key configuration
    And the following virtual keys exist:
      | key          | name    | allowed_models     | rpm_limit | tpm_limit | monthly_budget_cents | budget_mode | provider_key |
      | hub-team-a   | Team A  | gpt-4o, gpt-4o-mini| 60        | 100000    | 5000                 | hard        | openai       |
      | hub-team-b   | Team B  |                    | 120       | 200000    | 10000                | soft        | anthropic    |
      | hub-unlimited| VIP     |                    |           |           |                      | hard        | openai       |

  Scenario: Virtual key authentication succeeds
    Given a valid virtual key "hub-team-a"
    When I send a chat completion request with Authorization header "Bearer hub-team-a"
    Then the request should be authenticated
    And the response status should be 200

  Scenario: Virtual key authentication fails with invalid key
    Given an invalid virtual key "hub-invalid-key"
    When I send a chat completion request with Authorization header "Bearer hub-invalid-key"
    Then the response status should be 401

  Scenario: Virtual key authentication fails with disabled key
    Given a disabled virtual key "hub-disabled-key"
    When I send a chat completion request with Authorization header "Bearer hub-disabled-key"
    Then the response status should be 403

  Scenario: Model allowlist enforcement
    Given virtual key "hub-team-a" has allowed_models "gpt-4o, gpt-4o-mini"
    When I send a chat completion request with model "gpt-4o" and Authorization header "Bearer hub-team-a"
    Then the request should be authenticated
    And the response status should be 200

  Scenario: Model allowlist blocks unauthorized models
    Given virtual key "hub-team-a" has allowed_models "gpt-4o, gpt-4o-mini"
    When I send a chat completion request with model "claude-3-5-sonnet" and Authorization header "Bearer hub-team-a"
    Then the response status should be 403
    And the error message should contain "model not allowed"

  Scenario: Empty allowlist permits all models
    Given virtual key "hub-team-b" has empty allowed_models
    When I send a chat completion request with model "gpt-4o" and Authorization header "Bearer hub-team-b"
    Then the request should be authenticated
    And the response status should be 200

  Scenario: Rate limiting enforcement - RPM
    Given virtual key "hub-team-a" has rpm_limit 2
    When I send 3 chat completion requests within 1 minute with Authorization header "Bearer hub-team-a"
    Then the first 2 requests should succeed
    And the third request should have status 429
    And the response should include header "X-RateLimit-Remaining" with value "0"

  Scenario: Rate limiting enforcement - TPM
    Given virtual key "hub-team-a" has tpm_limit 1000
    When I send chat completion requests that consume 1000 tokens with Authorization header "Bearer hub-team-a"
    And I send another chat completion request
    Then the response status should be 429
    And the response should include header "X-RateLimit-Remaining" with value "0"

  Scenario: Budget enforcement - hard mode blocks requests
    Given virtual key "hub-team-a" has monthly_budget_cents 100 and budget_mode "hard"
    And the key has consumed 100 cents this month
    When I send a chat completion request with Authorization header "Bearer hub-team-a"
    Then the response status should be 402
    And the error message should contain "budget exceeded"

  Scenario: Budget enforcement - soft mode allows requests
    Given virtual key "hub-team-b" has monthly_budget_cents 100 and budget_mode "soft"
    And the key has consumed 100 cents this month
    When I send a chat completion request with Authorization header "Bearer hub-team-b"
    Then the response status should be 200
    And a warning log should be emitted for budget exceeded

  Scenario: Monthly budget reset
    Given virtual key "hub-team-a" has monthly_budget_cents 100 and budget_mode "hard"
    And the key has consumed 100 cents last month
    When I send a chat completion request with Authorization header "Bearer hub-team-a"
    Then the request should be authenticated
    And the response status should be 200

  Scenario: Unlimited budget key
    Given virtual key "hub-unlimited" has no monthly_budget_cents
    When I send a chat completion request with Authorization header "Bearer hub-unlimited"
    Then the request should be authenticated
    And the response status should be 200

  Scenario: Hot-reload adds new virtual key
    Given the gateway is running
    When I add a new virtual key "hub-new-team" via the management API
    And I wait for config reload
    When I send a chat completion request with Authorization header "Bearer hub-new-team"
    Then the request should be authenticated
    And the response status should be 200

  Scenario: Hot-reload disables virtual key
    Given the gateway is running with virtual key "hub-team-a" enabled
    When I disable virtual key "hub-team-a" via the management API
    And I wait for config reload
    When I send a chat completion request with Authorization header "Bearer hub-team-a"
    Then the response status should be 403
